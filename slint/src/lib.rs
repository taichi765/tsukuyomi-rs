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
use crate::fader_view_bridge::{adapt_2d_preview, adapt_fader_view};
use crate::preview_plugin::PreviewOutput;
// TODO: tsukuyomi_core::prelude使いたい

slint::include_modules!();

pub fn run_main() -> Result<(), Box<dyn Error>> {
    let mut doc = Doc::new();
    let mut command_manager = CommandManager::new();

    let (commands, scene_id) = create_some_presets();
    commands
        .into_iter()
        .for_each(|cmd| command_manager.execute(cmd, &mut doc).unwrap());

    let (command_tx, command_rx) = mpsc::channel::<EngineCommand>();
    let (error_tx, error_rx) = mpsc::channel::<EngineMessage>();

    let doc_event_bridge: Arc<RwLock<dyn DocObserver>> =
        Arc::new(RwLock::new(DocEventBridge::new(command_tx.clone())));
    doc.subscribe(Arc::downgrade(&doc_event_bridge));

    let doc = Arc::new(RwLock::new(doc));
    let engine = Engine::new(ReadOnly::new(Arc::clone(&doc)), command_rx, error_tx);

    let engine_handle = thread::Builder::new()
        .name("tsukuyomi-engine".into())
        .spawn(move || engine.start_loop())
        .unwrap();

    try_some_commands(command_tx.clone(), scene_id);

    // TODO: シリアライズからの復元
    let plugin = PreviewOutput::new();
    if let Err(e) = command_manager.execute(
        Box::new(doc_commands::AddOutput::new(
            UniverseId::new(0),
            plugin.id(),
        )),
        &mut doc.write().unwrap(),
    ) {
        println!("{}", e); // TODO: some error handling
    }
    command_tx
        .send(EngineCommand::AddPlugin(Box::new(plugin)))
        .unwrap();

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

    #[cfg(target_os = "macos")]
    ui.set_is_macos(true);
    #[cfg(not(target_os = "macos"))]
    ui.set_is_macos(false);

    adapt_fader_view(&ui, command_tx.clone());
    adapt_2d_preview(&ui);

    ui.run()?;

    if let Err(e) = command_tx.send(EngineCommand::Shutdown) {
        eprintln!("failed to send message to engine:{}", e);
    }

    engine_handle.join().unwrap();

    Ok(())
}

pub fn create_some_presets() -> (Vec<Box<dyn DocCommand>>, FunctionId) {
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

pub fn try_some_commands(command_tx: Sender<EngineCommand>, scene_id: FunctionId) {
    command_tx.send(EngineCommand::AddUniverse).unwrap();

    command_tx
        .send(EngineCommand::StartFunction(scene_id))
        .unwrap();
    thread::sleep(Duration::from_secs(1));
    command_tx
        .send(EngineCommand::StopFunction(scene_id))
        .unwrap();
}
