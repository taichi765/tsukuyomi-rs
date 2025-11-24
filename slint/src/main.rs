// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
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

    let mut doc = Doc::new();
    let mut command_manager = CommandManager::new();

    let (commands, scene_id) = create_some_presets();
    commands
        .into_iter()
        .for_each(|cmd| command_manager.execute(cmd, &mut doc).unwrap());

    let (command_tx, command_rx) = mpsc::channel::<EngineCommand>();
    let (error_tx, error_rx) = mpsc::channel::<EngineMessage>();

    let mut doc_event_bridge: Arc<RwLock<dyn DocObserver>> =
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

    //ui.on_request_save(|| println!("save requested"));

    /*ui.on_request_increase_value({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_counter(ui.get_counter() + 1);
        }
    });*/

    ui.run()?;

    Ok(())
}
