use std::sync::{Arc, RwLock};

use super::helpers::TestObserver;
use super::helpers::{make_fixture, make_fixture_def_with_mode};
use crate::{
    doc::{Doc, DocEvent, DocObserver, ResolveError},
    fixture::MergeMode,
    universe::{DmxAddress, UniverseId},
};

#[test]
fn insert_fixture_returns_none_then_some_and_emits_event() {
    let mut doc = Doc::new();

    // subscribe observer
    let observer = Arc::new(RwLock::new(TestObserver::new()));
    {
        let obs: Arc<RwLock<dyn DocObserver>> = Arc::clone(&observer) as _;
        doc.subscribe(Arc::downgrade(&obs));
    }

    // prepare dependencies: fixture def + universe
    let def = make_fixture_def_with_mode("ModelX", "ModeA", "Dimmer", 0, MergeMode::LTP);
    let def_id = def.id();
    doc.insert_fixture_def(def);

    let uni_id = UniverseId::new(1);
    doc.add_universe(uni_id);

    // create fixture
    let fxt = make_fixture("Fxt1", def_id, uni_id, DmxAddress::new(1).unwrap(), "ModeA");
    let fxt_id = fxt.id();

    // first insert -> None
    let old = doc.insert_fixture(fxt.clone());
    assert!(old.is_none());

    // event emitted
    {
        let obs = observer.read().unwrap();
        assert!(
            obs.events
                .iter()
                .any(|e| matches!(e, DocEvent::FixtureInserted(id) if *id == fxt_id))
        );
    }

    // second insert with same id -> Some(previous)
    let old2 = doc.insert_fixture(fxt);
    assert!(old2.is_some());
    assert_eq!(old2.unwrap().id(), fxt_id);

    // event emitted again for insertion
    {
        let obs = observer.read().unwrap();
        let count = obs
            .events
            .iter()
            .filter(|e| matches!(e, DocEvent::FixtureInserted(id) if *id == fxt_id))
            .count();
        assert!(count >= 2);
    }
}

#[test]
fn remove_fixture_returns_some_then_none_and_emits_event() {
    let mut doc = Doc::new();

    // subscribe observer
    let observer = Arc::new(RwLock::new(TestObserver::new()));
    {
        let obs: Arc<RwLock<dyn DocObserver>> = Arc::clone(&observer) as _;
        doc.subscribe(Arc::downgrade(&obs));
    }

    // prepare dependencies: fixture def + universe
    let def = make_fixture_def_with_mode("ModelX", "ModeA", "Dimmer", 0, MergeMode::HTP);
    let def_id = def.id();
    doc.insert_fixture_def(def);

    let uni_id = UniverseId::new(1);
    doc.add_universe(uni_id);

    // insert fixture
    let fxt = make_fixture(
        "Fxt2",
        def_id,
        uni_id,
        DmxAddress::new(10).unwrap(),
        "ModeA",
    );
    let fxt_id = fxt.id();
    doc.insert_fixture(fxt);

    // remove once -> Some
    let removed = doc.remove_fixture(&fxt_id);
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().id(), fxt_id);

    // event emitted
    {
        let obs = observer.read().unwrap();
        assert!(
            obs.events
                .iter()
                .any(|e| matches!(e, DocEvent::FixtureRemoved(id) if *id == fxt_id))
        );
    }

    // remove again -> None
    assert!(doc.remove_fixture(&fxt_id).is_none());
}

#[test]
fn resolve_address_fails_after_fixture_removed() {
    let mut doc = Doc::new();

    // prepare dependencies: fixture def + universe
    let def = make_fixture_def_with_mode("ModelX", "ModeA", "Dimmer", 5, MergeMode::LTP);
    let def_id = def.id();
    doc.insert_fixture_def(def);

    let uni_id = UniverseId::new(1);
    doc.add_universe(uni_id);

    // insert fixture
    let base = 20;
    let fxt = make_fixture(
        "Fxt1",
        def_id,
        uni_id,
        DmxAddress::new(base).unwrap(),
        "ModeA",
    );
    let fxt_id = fxt.id();
    doc.insert_fixture(fxt);

    // resolve works before removal
    let (resolved_uni, resolved) = doc.resolve_address(fxt_id, "Dimmer").unwrap();
    assert_eq!(resolved_uni, uni_id);
    assert_eq!(resolved.address.value(), base + 5);

    // remove and then resolution should fail
    doc.remove_fixture(&fxt_id);
    let err = doc.resolve_address(fxt_id, "Dimmer").err().unwrap();
    assert!(matches!(err, ResolveError::FixtureNotFound(id) if id == fxt_id));
}
