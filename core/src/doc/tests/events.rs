use std::sync::{Arc, RwLock};

use super::helpers::{TestObserver, make_fixture, make_fixture_def_with_mode, make_function};
use crate::{
    doc::{Doc, DocEvent, DocObserver},
    engine::OutputPluginId,
    fixture::MergeMode,
    universe::{DmxAddress, UniverseId},
};

// Utility: find index of the first event matching predicate at or after `start_at`
fn find_event_idx<F>(events: &[DocEvent], start_at: usize, pred: F) -> Option<usize>
where
    F: Fn(&DocEvent) -> bool,
{
    events
        .iter()
        .enumerate()
        .skip(start_at)
        .find_map(|(i, e)| if pred(e) { Some(i) } else { None })
}

#[test]
fn events_sequence_contains_expected_order() {
    let mut doc = Doc::new();

    let observer = Arc::new(RwLock::new(TestObserver::new()));
    {
        let obs: Arc<RwLock<dyn DocObserver>> = Arc::clone(&observer) as _;
        doc.subscribe(Arc::downgrade(&obs));
    }

    // 1) Insert FixtureDef
    let def = make_fixture_def_with_mode("ModelX", "ModeA", "Dimmer", 0, MergeMode::LTP);
    let def_id = def.id();
    doc.insert_fixture_def(def);

    // 2) Add Function
    let func = make_function("Func1");
    let func_id = func.id();
    doc.add_function(func);

    // 3) Add Universe
    let uni_id = UniverseId::new(1);
    doc.add_universe(uni_id);

    // 4) Insert Fixture
    let fxt = make_fixture("Fx1", def_id, uni_id, DmxAddress::new(1).unwrap(), "ModeA");
    let fxt_id = fxt.id();
    doc.insert_fixture(fxt);

    // 5) Add Output (emits UniverseSettingsChanged)
    let plugin_id = OutputPluginId::new();
    doc.add_output(uni_id, plugin_id).unwrap();

    // 6) Remove Fixture
    doc.remove_fixture(&fxt_id);

    // 7) Remove FixtureDef
    doc.remove_fixture_def(&def_id);

    // 8) Remove Universe
    doc.remove_universe(&uni_id);

    let events = observer.read().unwrap().events.clone();

    // Verify subsequence presence (order matters, adjacency does not)
    let mut cur = 0usize;

    cur = find_event_idx(
        &events,
        cur,
        |e| matches!(e, DocEvent::FixtureDefInserted(id) if *id == def_id),
    )
    .expect("FixtureDefInserted not found")
        + 1;

    cur = find_event_idx(
        &events,
        cur,
        |e| matches!(e, DocEvent::FunctionInserted(id) if *id == func_id),
    )
    .expect("FunctionInserted not found")
        + 1;

    cur = find_event_idx(
        &events,
        cur,
        |e| matches!(e, DocEvent::UniverseAdded(id) if *id == uni_id),
    )
    .expect("UniverseAdded not found")
        + 1;

    cur = find_event_idx(
        &events,
        cur,
        |e| matches!(e, DocEvent::FixtureInserted(id) if *id == fxt_id),
    )
    .expect("FixtureInserted not found")
        + 1;

    cur = find_event_idx(&events, cur, |e| {
        matches!(e, DocEvent::UniverseSettingsChanged)
    })
    .expect("UniverseSettingsChanged not found")
        + 1;

    cur = find_event_idx(
        &events,
        cur,
        |e| matches!(e, DocEvent::FixtureRemoved(id) if *id == fxt_id),
    )
    .expect("FixtureRemoved not found")
        + 1;

    cur = find_event_idx(
        &events,
        cur,
        |e| matches!(e, DocEvent::FixtureDefRemoved(id) if *id == def_id),
    )
    .expect("FixtureDefRemoved not found")
        + 1;

    let _ = find_event_idx(
        &events,
        cur,
        |e| matches!(e, DocEvent::UniverseRemoved(id) if *id == uni_id),
    )
    .expect("UniverseRemoved not found");
}
