use std::{
    collections::HashMap,
    sync::{Arc, mpsc},
    thread,
    time::Duration,
};

use tsukuyomi_core::{
    commands::{self, CommandManager},
    doc::Doc,
    engine::{Engine, EngineCommand},
    fixture::{Fixture, MergeMode},
    fixture_def::{ChannelDef, FixtureDef, FixtureMode},
    functions::{FunctionData, FunctionDataGetters, SceneValue, StaticSceneData},
    plugins::Plugin,
    universe::{DmxAddress, UniverseId},
};

struct MockPlugin {}

impl Plugin for MockPlugin {
    fn send_dmx(&self, universe_id: u8, dmx_data: &[u8]) -> Result<(), std::io::Error> {
        println!("{universe_id}: {}", dmx_data[0]);
        Ok(())
    }
}

#[test]
fn engine_can_start_function() {
    let mut doc = Doc::new();
    let mut command_manager = CommandManager::new();

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
    command_manager
        .execute(
            Box::new(commands::doc::AddFixtureDef::new(fixture_def)),
            &mut doc,
        )
        .unwrap();

    let fixture = Fixture::new(
        "Fixture",
        UniverseId::new(0),
        DmxAddress::new(0).unwrap(),
        fixture_def_id,
        "Mode 1".into(),
    );

    let fixture_id = fixture.id();
    command_manager
        .execute(Box::new(commands::doc::AddFixture::new(fixture)), &mut doc)
        .unwrap();

    let mut scene = StaticSceneData::new("My Scene");
    let mut sv = SceneValue::new();
    sv.insert("Dimmer".into(), 200);
    sv.insert("Red".into(), 100);
    scene.insert_value(fixture_id, sv);
    let scene_id = scene.id();

    command_manager
        .execute(
            Box::new(commands::doc::AddFunction::new(FunctionData::StaticScene(
                scene,
            ))),
            &mut doc,
        )
        .unwrap();

    let (command_tx, command_rx) = mpsc::channel::<EngineCommand>();
    let engine = Engine::new(Arc::new(doc), command_rx);

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
}
