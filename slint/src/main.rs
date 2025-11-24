// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use std::sync::{mpsc, Arc, RwLock};
use std::thread;
use std::time::Duration;
use tsukuyomi_core::commands;
use tsukuyomi_core::doc::Doc;
use tsukuyomi_core::engine::{Engine, EngineCommand, EngineMessage};
use tsukuyomi_core::functions::{FunctionData, StaticSceneData};
use tsukuyomi_core::readonly::ReadOnly;
use tsukuyomi_slint::command_manager::CommandManager;
use tsukuyomi_slint::create_some_presets;

slint::include_modules!();

struct MockPlugin {}

impl Plugin for MockPlugin {
    fn send_dmx(&self, universe_id: u8, dmx_data: &[u8]) -> Result<(), std::io::Error> {
        println!("{universe_id}: {}", dmx_data[0]);
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    let mut doc = Doc::new();
    let mut command_manager = CommandManager::new();

    let (commands,scene_id)=create_some_presets();
    commands.into_iter().for_each(|cmd|command_manager.execute(cmd,&mut doc).unwrap());

    let doc = Arc::new(RwLock::new(doc));

    let (command_tx, command_rx) = mpsc::channel::<EngineCommand>();
    let (error_tx, error_rx) = mpsc::channel::<EngineMessage>();
    let engine = Engine::new(ReadOnly::new(Arc::clone(&doc)), command_rx, error_tx);

    let engine_handle = thread::Builder::new()
        .name("tsukuyomi-engine".into())
        .spawn(move || engine.start_loop())
        .unwrap();

    command_tx.send(EngineCommand::AddUniverse).unwrap();

    command_tx
        .send(EngineCommand::AddPlugin(Box::new(MockPlugin {})))
        .unwrap();

    command_tx
        .send(EngineCommand::StartFunction(scene_id))
        .unwrap();
    thread::sleep(Duration::from_secs(1));
    command_tx
        .send(EngineCommand::StopFunction(scene_id))
        .unwrap();
    command_tx.send(EngineCommand::Shutdown).unwrap();

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
