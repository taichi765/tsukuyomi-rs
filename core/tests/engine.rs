use std::{
    sync::{Arc, mpsc},
    thread,
    time::Duration,
};

use tsukuyomi_core::{
    doc::Doc,
    engine::{Engine, EngineCommand},
    functions::{FunctionDataGetters, SceneValue, StaticSceneData},
};
use uuid::Uuid;

#[test]
fn engine_can_start_function() {
    let mut doc = Doc::new();
    let fixture_id = Uuid::new_v4();

    let mut scene = StaticSceneData::new("My Scene");
    let mut sv = SceneValue::new();
    sv.insert("Dimmer".into(), 200);
    sv.insert("Red".into(), 100);
    scene.insert_value(fixture_id, sv);

    let scene_id = scene.id();

    //doc.add_function(FunctionData::StaticScene(scene)).unwrap();

    let (command_tx, command_rx) = mpsc::channel::<EngineCommand>();
    let mut engine = Engine::new(Arc::new(doc), command_rx);

    let engine_handle = thread::Builder::new()
        .name("tsukuyomi-engine".into())
        .spawn(move || engine.start_loop())
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
}
