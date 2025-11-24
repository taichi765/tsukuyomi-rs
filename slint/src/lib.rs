use std::collections::HashMap;

use tsukuyomi_core::functions::FunctionId;
use tsukuyomi_core::{
    commands,
    doc::DocCommand,
    fixture::{Fixture, MergeMode},
    fixture_def::{ChannelDef, FixtureDef, FixtureMode},
    functions::{FunctionData, FunctionDataGetters, SceneValue, StaticSceneData},
    universe::{DmxAddress, UniverseId},
};

pub mod doc_event_bridge;
// TODO: tsukuyomi_core::prelude使いたい

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
    commands.push(Box::new(commands::doc::AddFixtureDef::new(fixture_def)));

    let fixture = Fixture::new(
        "Fixture",
        UniverseId::new(0),
        DmxAddress::new(0).unwrap(),
        fixture_def_id,
        "Mode 1".into(),
    );

    let fixture_id = fixture.id();
    commands.push(Box::new(commands::doc::AddFixture::new(fixture)));

    let mut scene = StaticSceneData::new("My Scene");
    let mut sv = SceneValue::new();
    sv.insert("Dimmer".into(), 200);
    sv.insert("Red".into(), 100);
    scene.insert_value(fixture_id, sv);
    let scene_id = scene.id();

    commands.push(Box::new(commands::doc::AddFunction::new(
        FunctionData::StaticScene(scene),
    )));

    (commands, scene_id)
}
