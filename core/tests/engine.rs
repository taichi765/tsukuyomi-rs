use std::{
    sync::{Arc, RwLock, mpsc},
    thread,
    time::Duration,
};

use tsukuyomi_core::{
    doc::Doc,
    engine::{Engine, EngineCommand},
    functions::{FunctionData, SceneValue, StaticSceneData},
};

#[test]
fn engine_can_start_function() {
    let mut doc = Doc::new();

    let mut scene = StaticSceneData::new(0, "My Scene");
    let mut sv1 = SceneValue::new();
    sv1.insert(1, 10);
    scene.insert_value(1, sv1);

    let mut sv2 = SceneValue::new();
    sv2.insert(5, 200);
    sv2.insert(6, 201);
    scene.insert_value(2, sv2);

    //doc.add_function(FunctionData::StaticScene(scene)).unwrap();

    let (command_tx, command_rx) = mpsc::channel::<EngineCommand>();
    let mut engine = Engine::new(Arc::new(RwLock::new(doc)), command_rx);

    let engine_handle = thread::Builder::new()
        .name("tsukuyomi-engine".into())
        .spawn(move || engine.start_loop())
        .unwrap();

    command_tx.send(EngineCommand::StartFunction(0)).unwrap();
    thread::sleep(Duration::from_secs(1));
    command_tx.send(EngineCommand::StopFunction(0)).unwrap();
    command_tx.send(EngineCommand::Shutdown).unwrap();

    engine_handle.join().unwrap();
}
