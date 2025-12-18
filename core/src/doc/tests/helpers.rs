use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::{Arc, RwLock},
};

use crate::{
    doc::{DocEvent, DocEventBus, DocHandle, DocObserver, DocStore},
    fixture::{Fixture, MergeMode},
    fixture_def::{ChannelDef, ChannelKind, FixtureDef, FixtureDefId, FixtureMode},
    functions::{FunctionData, StaticSceneData},
    universe::{DmxAddress, UniverseId},
};

pub(crate) struct TestObserver {
    pub events: Vec<DocEvent>,
}
impl TestObserver {
    pub(crate) fn new() -> Self {
        Self { events: vec![] }
    }
}
impl DocObserver for TestObserver {
    fn on_doc_event(&mut self, event: &DocEvent) {
        self.events.push(event.clone());
    }
}

/// Creates a new DocHandle with an observer already subscribed.
/// Returns the DocHandle, the underlying DocStore, and the observer for event verification.
pub(crate) fn make_doc_handle_with_observer()
-> (DocHandle, Arc<RwLock<DocStore>>, Arc<RwLock<TestObserver>>) {
    let doc_store = Arc::new(RwLock::new(DocStore::new()));
    let mut event_bus = DocEventBus::new();

    let observer = Arc::new(RwLock::new(TestObserver::new()));
    let obs: Arc<RwLock<dyn DocObserver>> = Arc::clone(&observer) as _;
    event_bus.subscribe(Arc::downgrade(&obs));

    let handle = DocHandle::new(Arc::clone(&doc_store), Rc::new(RefCell::new(event_bus)));
    (handle, doc_store, observer)
}

/// Build a minimal FixtureDef with a single mode and a single channel entry.
/// - manufacturer is fixed to "TestMfr"
/// - model is provided via `model`
/// - the mode `mode_name` contains `channel_name` at `channel_offset` with `merge_mode`
pub(crate) fn make_fixture_def_with_mode(
    model: &str,
    mode_name: &str,
    channel_name: &str,
    channel_offset: usize,
    merge_mode: MergeMode,
    kind: ChannelKind,
) -> FixtureDef {
    let mut def = FixtureDef::new("TestMfr".to_string(), model.to_string());

    def.insert_channel(
        String::from(channel_name),
        ChannelDef::new(merge_mode, kind),
    );

    let channel_order = vec![(String::from(channel_name), channel_offset)];

    let mode = FixtureMode::new(channel_order.into_iter()).unwrap();
    def.insert_mode(String::from(mode_name), mode);

    def
}

pub(crate) fn make_def_with_two_channels() -> FixtureDef {
    // Manufacturer/Model arbitrary for test
    let mut def = FixtureDef::new("TestMfr", "ModelDual");

    // Insert two channel templates: Dimmer (offset 0) and Color (offset 3)
    def.insert_channel(
        "Dimmer",
        ChannelDef::new(crate::fixture::MergeMode::LTP, ChannelKind::Dimmer),
    );
    def.insert_channel(
        "Color",
        ChannelDef::new(crate::fixture::MergeMode::HTP, ChannelKind::Red),
    );

    // Mode order specifies offsets
    let order = vec![("Dimmer".to_string(), 0), ("Color".to_string(), 1)];
    let mode = FixtureMode::new(order.into_iter()).unwrap();
    def.insert_mode("ModeA", mode);

    def
}

/// Build a Fixture that references a given FixtureDef and mode.
pub(crate) fn make_fixture(
    name: &str,
    fixture_def_id: FixtureDefId,
    universe_id: UniverseId,
    address: DmxAddress,
    fixture_mode: &str,
) -> Fixture {
    Fixture::new(
        name,
        universe_id,
        address,
        fixture_def_id,
        String::from(fixture_mode),
    )
}

/// Build a simple FunctionData (StaticScene) with the given name.
pub(crate) fn make_function(name: &str) -> FunctionData {
    FunctionData::StaticScene(StaticSceneData::new(name))
}
