use std::{
    collections::HashMap,
    rc::Rc,
    sync::{
        Arc, Mutex, RwLock,
        mpsc::{self, Receiver, Sender},
    },
};

use slint::{Brush, Color, ComponentHandle, Model, ModelRc, ToSharedString, VecModel, Weak};
use tracing::{debug, debug_span};
use tsukuyomi_core::{
    command_manager::CommandManager,
    commands::doc_commands,
    doc::{Doc, DocEvent, DocObserver},
    engine::{EngineCommand, OutputPluginId},
    fixture::{Fixture, FixtureId},
    fixture_def::ChannelKind,
    plugins::Plugin,
    readonly::ReadOnly,
    universe::{DmxAddress, UniverseId},
};
use uuid::Uuid;

use crate::{AppWindow, FixtureEntityData, Preview2DLogic};

/// Returns closure to update preview, which should be called in [slint::Timer]
pub fn setup_2d_preview(
    ui: &AppWindow,
    doc: Arc<RwLock<Doc>>,
    command_manager: &mut CommandManager,
    command_tx: Sender<EngineCommand>,
) -> impl FnMut() -> () + 'static {
    let (msg_tx, msg_rx) = mpsc::channel::<PreviewMessage>();
    let controller = Arc::new(RwLock::new(PreviewController::new(
        ReadOnly::new(Arc::clone(&doc)),
        ui.as_weak(),
        Mutex::new(msg_rx),
    )));

    doc.write()
        .unwrap()
        .subscribe(Arc::downgrade(&controller) as _);

    // TODO: シリアライズからの復元
    let plugin = PreviewPlugin::new(msg_tx);
    let p_id = plugin.id();

    command_tx
        .send(EngineCommand::AddPlugin(Box::new(plugin)))
        .unwrap();
    command_manager
        .execute(
            Box::new(doc_commands::AddOutput::new(
                UniverseId::new(1), //FIXME: hard coding
                p_id,
            )),
            &mut doc.write().unwrap(),
        )
        .unwrap();

    // Reset dummy properties
    ui.global::<Preview2DLogic>()
        .set_fixture_list(Rc::new(VecModel::from(Vec::new())).into());
    return move || {
        controller.write().unwrap().handle_messages();
    };
}

/// Just sends message to [PreviewController] when [`Engine`][tsukuyomi_core::engine::Engine] ticked
struct PreviewPlugin {
    id: OutputPluginId,
    msg_tx: Sender<PreviewMessage>,
}

impl PreviewPlugin {
    pub fn new(msg_tx: Sender<PreviewMessage>) -> Self {
        Self {
            id: OutputPluginId::new(),
            msg_tx,
        }
    }
}

impl Plugin for PreviewPlugin {
    fn send_dmx(&self, universe_id: UniverseId, dmx_data: &[u8]) -> Result<(), std::io::Error> {
        self.msg_tx
            .send(PreviewMessage::DmxFrame {
                universe_id,
                dmx_data: dmx_data.to_owned(), // FIXME: 参照のまま渡せないか？
            })
            .expect("failed to send message from preview plugin to preview controller");
        Ok(())
    }

    fn id(&self) -> OutputPluginId {
        self.id
    }
}

/// Controls actual Preview UI
struct PreviewController {
    doc: ReadOnly<Doc>,
    ui_handle: Weak<AppWindow>,
    msg_rx: Mutex<Receiver<PreviewMessage>>,
}

impl PreviewController {
    fn new(
        doc: ReadOnly<Doc>,
        ui_handle: Weak<AppWindow>,
        msg_rx: Mutex<Receiver<PreviewMessage>>,
    ) -> Self {
        Self {
            doc,
            ui_handle,
            msg_rx,
        }
    }

    fn update_fixture_map(&mut self, id: FixtureId, fixture: &Fixture) {
        let _span = debug_span!("preview: update_fixture_map").entered();
        debug!("called");
        debug!("got read lock");
        let ui = self.ui_handle.unwrap();
        let mut fixtures: HashMap<FixtureId, FixtureEntityData> =
            modelrc_to_map(ui.global::<Preview2DLogic>().get_fixture_list());

        fixtures.insert(
            id,
            FixtureEntityData {
                color: Brush::SolidColor(Color::from_rgb_u8(0, 0, 0)),
                fixture_id: id.to_shared_string(),
                x: 500., //TODO
                y: 220., //TODO
            },
        );
        let fixtures_model: Vec<FixtureEntityData> = fixtures.into_iter().map(|(_, v)| v).collect();
        ui.global::<Preview2DLogic>()
            .set_fixture_list(Rc::new(VecModel::from(fixtures_model)).into());
    }

    fn handle_messages(&mut self) {
        // TODO
        while let Ok(msg) = self.msg_rx.lock().unwrap().try_recv() {
            match msg {
                PreviewMessage::DmxFrame {
                    universe_id,
                    dmx_data,
                } => self.apply_dmx_frame(universe_id, dmx_data),
            }
        }
    }

    fn apply_dmx_frame(&self, universe_id: UniverseId, dmx_data: Vec<u8>) {
        let mut fixture_color_map: HashMap<FixtureId, (u8, u8, u8, u8)> = HashMap::new();

        let doc = self.doc.read();
        for (address, value) in dmx_data.iter().enumerate() {
            let Some(&(fixture_id, offset)) = self
                .doc
                .read()
                .get_fixture_by_address(&universe_id, DmxAddress::new(address).unwrap())
            //FIXME: キャッシュ
            else {
                continue;
            };
            let fixture = doc.get_fixture(&fixture_id).unwrap();
            let def = doc.get_fixture_def(&fixture.fixture_def()).unwrap();
            let mode = def.modes().get(fixture.fixture_mode()).unwrap();
            let channel_name = mode.get_channel_by_offset(offset).unwrap();
            let chanel = def.channel_templates().get(channel_name).unwrap();

            match chanel.kind() {
                ChannelKind::Dimmer => set_color(fixture_id, &mut fixture_color_map, 0, *value),
                ChannelKind::Red => set_color(fixture_id, &mut fixture_color_map, 1, *value),
                ChannelKind::Blue => set_color(fixture_id, &mut fixture_color_map, 2, *value),
                ChannelKind::Green => set_color(fixture_id, &mut fixture_color_map, 3, *value),
                ChannelKind::White => todo!(),
            }
        }
        let ui = self.ui_handle.unwrap();

        let mut fixture_map = modelrc_to_map(ui.global::<Preview2DLogic>().get_fixture_list());
        for (id, (dimmer, r, g, b)) in fixture_color_map {
            let data = fixture_map.get_mut(&id).unwrap();
            data.color = calc_color(dimmer, r, g, b);
        }
        let fixture_vec: Vec<FixtureEntityData> =
            fixture_map.into_iter().map(|(_, data)| data).collect();
        println!("{:?}", fixture_vec);
        ui.global::<Preview2DLogic>()
            .set_fixture_list(Rc::new(VecModel::from(fixture_vec)).into());
    }
}

impl DocObserver for PreviewController {
    fn on_doc_event(&mut self, event: &DocEvent) {
        debug!("event recieved");
        match event {
            DocEvent::FixtureInserted(id, fixture) => {
                self.update_fixture_map(*id, fixture);
            }
            _ => (),
        }
    }
}

enum PreviewMessage {
    DmxFrame {
        universe_id: UniverseId,
        dmx_data: Vec<u8>,
    },
}

fn set_color(
    fixture_id: FixtureId,
    map: &mut HashMap<FixtureId, (u8, u8, u8, u8)>,
    index: usize,
    value: u8,
) {
    debug!(index, value);
    if let Some(color) = map.get_mut(&fixture_id) {
        match index {
            0 => color.0 = value,
            1 => color.1 = value,
            2 => color.2 = value,
            3 => color.3 = value,
            _ => (),
        }
    } else {
        let mut new_color = (255, 255, 255, 255);
        match index {
            0 => new_color.0 = value,
            1 => new_color.1 = value,
            2 => new_color.2 = value,
            3 => new_color.3 = value,
            _ => (),
        }
        map.insert(fixture_id, new_color);
    }
}

fn calc_color(dimmer: u8, r: u8, g: u8, b: u8) -> Brush {
    let ratio = dimmer as f32 / 255 as f32;
    let r = (r as f32 * ratio) as u8;
    let g = (g as f32 * ratio) as u8;
    let b = (b as f32 * ratio) as u8;
    debug!(dimmer, r, g, b);
    Brush::SolidColor(Color::from_rgb_u8(r, g, b))
}

fn modelrc_to_map(value: ModelRc<FixtureEntityData>) -> HashMap<FixtureId, FixtureEntityData> {
    value
        .iter()
        .map(|f| {
            (
                FixtureId::from(Uuid::parse_str(f.fixture_id.as_str()).unwrap()),
                f,
            )
        })
        .collect()
}
