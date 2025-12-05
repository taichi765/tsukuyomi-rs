use std::{
    rc::Rc,
    sync::{Arc, RwLock, mpsc::Sender},
};

use crate::{
    AppWindow, FaderLogic, FixtureEntityData, Preview2DLogic, preview_plugin::PreviewOutput,
};
use slint::{Brush, Color, ComponentHandle, SharedString, VecModel};
use tracing::trace;
use tsukuyomi_core::{
    command_manager::CommandManager, commands::doc_commands, doc::Doc, engine::EngineCommand,
    fixture::FixtureId, plugins::Plugin, universe::UniverseId,
};
use uuid::Uuid;

pub fn setup_fader_view(ui: &AppWindow, command_tx: Sender<EngineCommand>) {
    ui.global::<FaderLogic>().on_value_changed(
        move |_fixture_id: SharedString, channel: SharedString, value: i32| {
            println!("value changed!");
            command_tx
                .send(EngineCommand::SetLiveValue {
                    fixture_id: FixtureId::from(Uuid::nil()),
                    channel: channel.to_string(),
                    value: value as u8,
                })
                .expect("failed to send command to engine");
        },
    );
}

pub fn setup_2d_preview(
    ui: &AppWindow,
    doc: Arc<RwLock<Doc>>,
    command_manager: &mut CommandManager,
    command_tx: Sender<EngineCommand>,
) {
    // TODO: シリアライズからの復元
    let plugin = PreviewOutput::new();
    let p_id = plugin.id();

    trace!("added plugin to engine");
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

    let fixture_list: Vec<FixtureEntityData> = vec![
        FixtureEntityData {
            x: 10.0,
            y: 10.0,
            color: Brush::SolidColor(Color::from_rgb_u8(255, 255, 0)),
        },
        FixtureEntityData {
            x: 50.0,
            y: 50.0,
            color: Brush::SolidColor(Color::from_rgb_u8(0, 255, 127)),
        },
    ];
    ui.global::<Preview2DLogic>()
        .set_fixture_list(Rc::new(VecModel::from(fixture_list)).into());
}
