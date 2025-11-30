// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use i_slint_backend_winit::WinitWindowAccessor;
use slint::{Brush, Color, VecModel};
use std::error::Error;
use std::rc::Rc;
use std::sync::{Arc, RwLock, mpsc};
use std::thread;
use tsukuyomi_core::command_manager::CommandManager;
use tsukuyomi_core::doc::{Doc, DocObserver};
use tsukuyomi_core::engine::{Engine, EngineCommand, EngineMessage};
use tsukuyomi_core::readonly::ReadOnly;
use tsukuyomi_slint::doc_event_bridge::DocEventBridge;
use tsukuyomi_slint::{create_some_presets, try_some_commands};

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;
    // TODO: language switch(preferences)
    slint::select_bundled_translation("en".into()).unwrap();

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

    try_some_commands(command_tx, scene_id);

    engine_handle.join().unwrap();

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
    let ui_handle = ui.as_weak();
    ui.on_close(move || {
        let ui = ui_handle.unwrap();
        ui.window().hide().unwrap()
    });

    ui.run()?;

    Ok(())
}
