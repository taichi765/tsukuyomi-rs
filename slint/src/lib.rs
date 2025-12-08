pub mod bottom_panel_bridge;
pub mod doc_event_bridge;
pub mod fader_view_bridge;
pub mod preview_plugin;

use std::collections::HashMap;
use std::error::Error;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock, Weak, mpsc};
use std::thread::{self, JoinHandle};

use i_slint_backend_winit::WinitWindowAccessor;

use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use tsukuyomi_core::command_manager::CommandManager;
use tsukuyomi_core::commands::doc_commands;
use tsukuyomi_core::engine::{Engine, EngineCommand, EngineMessage};
use tsukuyomi_core::functions::FunctionId;
use tsukuyomi_core::{
    commands,
    commands::DocCommand,
    doc::{Doc, DocObserver},
    fixture::{Fixture, MergeMode},
    fixture_def::{ChannelDef, FixtureDef, FixtureMode},
    functions::{FunctionData, FunctionDataGetters, SceneValue, StaticSceneData},
    readonly::ReadOnly,
    universe::{DmxAddress, UniverseId},
};

use crate::doc_event_bridge::DocEventBridge;
use crate::fader_view_bridge::{setup_2d_preview, setup_fader_view};
// TODO: tsukuyomi_core::prelude使いたい

slint::include_modules!();

pub fn run_main() -> Result<(), Box<dyn Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let doc = Arc::new(RwLock::new(Doc::new()));
    // HACK: Initialize engine and bridge before changing Doc.
    // If Doc is changed before engine initialized, number of universe would be
    // unsynchronized.
    let (engine_handle, command_tx, error_rx, _bridge) = setup_engine(Arc::clone(&doc));

    let mut command_manager = CommandManager::new();

    let (commands, scene_id) = create_some_presets();
    commands.into_iter().for_each(|cmd| {
        command_manager
            .execute(cmd, &mut doc.write().unwrap())
            .unwrap()
    });
    for i in 1..5 {
        command_manager
            .execute(
                Box::new(doc_commands::AddUniverse::new(UniverseId::new(i))),
                &mut doc.write().unwrap(),
            )
            .unwrap();
    }

    let ui = setup_window().expect("failed to setup ui");

    setup_fader_view(&ui, command_tx.clone());
    setup_2d_preview(
        &ui,
        Arc::clone(&doc),
        &mut command_manager,
        command_tx.clone(),
    );

    ui.run()?;

    if let Err(e) = command_tx.send(EngineCommand::Shutdown) {
        eprintln!("failed to send message to engine:{}", e);
    }

    engine_handle.join().unwrap();

    Ok(())
}

fn setup_engine(
    doc: Arc<RwLock<Doc>>,
) -> (
    JoinHandle<()>,
    Sender<EngineCommand>,
    Receiver<EngineMessage>,
    Arc<RwLock<DocEventBridge>>,
) {
    let (command_tx, command_rx) = mpsc::channel::<EngineCommand>();
    let (error_tx, error_rx) = mpsc::channel::<EngineMessage>();

    let bridge = Arc::new(RwLock::new(DocEventBridge::new(command_tx.clone())));
    let weak: Weak<RwLock<dyn DocObserver>> = Arc::downgrade(&bridge) as _;
    doc.write().unwrap().subscribe(weak);

    let engine = Engine::new(ReadOnly::new(doc), command_rx, error_tx);

    let engine_handle = thread::Builder::new()
        .name("tsukuyomi-engine".into())
        .spawn(move || engine.start_loop())
        .unwrap();
    (engine_handle, command_tx, error_rx, bridge)
}

fn setup_window() -> Result<AppWindow, Box<dyn Error>> {
    let ui = AppWindow::new()?;
    // TODO: language switch(preferences)
    slint::select_bundled_translation("en".into()).unwrap();

    let ui_handle = ui.as_weak();
    ui.on_start_drag(move || {
        let ui = ui_handle.unwrap();
        ui.window().with_winit_window(|w| w.drag_window());
    });

    let ui_handle = ui.as_weak();
    ui.on_minimize(move || {
        let ui = ui_handle.unwrap();
        ui.window().set_minimized(true);
    });

    // TODO: toggle-fullscreen

    let ui_handle = ui.as_weak();
    ui.on_close(move || {
        let ui = ui_handle.unwrap();
        ui.window().hide().unwrap()
    });

    Ok(ui)
}

fn create_some_presets() -> (Vec<Box<dyn DocCommand>>, FunctionId) {
    let mut commands: Vec<Box<dyn DocCommand>> = Vec::new();

    let mut fixture_def = FixtureDef::new("Test Manufacturer".into(), "Test Model".into());
    let mut mode: HashMap<String, Option<(usize, ChannelDef)>> = HashMap::new();
    mode.insert(
        "Dimmer".into(),
        Some((
            0,
            ChannelDef {
                merge_mode: MergeMode::HTP,
            },
        )),
    );
    mode.insert(
        "Red".into(),
        Some((
            1,
            ChannelDef {
                merge_mode: MergeMode::HTP,
            },
        )),
    );
    let mode = FixtureMode {
        channel_order: mode,
    };
    fixture_def.add_mode("Mode 1".into(), mode);
    let fixture_def_id = fixture_def.id();
    commands.push(Box::new(commands::doc_commands::AddFixtureDef::new(
        fixture_def,
    )));

    let fixture = Fixture::new(
        "Fixture",
        UniverseId::new(0),
        DmxAddress::new(0).unwrap(),
        fixture_def_id,
        "Mode 1".into(),
    );

    let fixture_id = fixture.id();
    commands.push(Box::new(commands::doc_commands::AddFixture::new(fixture)));

    let mut scene = StaticSceneData::new("My Scene");
    let mut sv = SceneValue::new();
    sv.insert("Dimmer".into(), 200);
    sv.insert("Red".into(), 100);
    scene.insert_value(fixture_id, sv);
    let scene_id = scene.id();

    commands.push(Box::new(commands::doc_commands::AddFunction::new(
        FunctionData::StaticScene(scene),
    )));

    (commands, scene_id)
}
