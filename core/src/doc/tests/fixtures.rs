use std::sync::{Arc, RwLock};

use super::helpers::TestObserver;
use super::helpers::{make_fixture, make_fixture_def_with_mode};
use crate::doc::{
    FixtureDefNotFound, FixtureInsertError, FixtureNotFound, FixtureRemoveError, ModeNotFound,
};
use crate::fixture_def::{ChannelKind, FixtureDefId};
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
    let def = make_fixture_def_with_mode(
        "ModelX",
        "ModeA",
        "Dimmer",
        0,
        MergeMode::LTP,
        ChannelKind::Dimmer,
    );
    let def_id = def.id();
    doc.insert_fixture_def(def);

    let uni_id = UniverseId::new(1);
    doc.add_universe(uni_id);

    // create fixture
    let fxt = make_fixture("Fxt1", def_id, uni_id, DmxAddress::new(1).unwrap(), "ModeA");
    let fxt_id = fxt.id();

    // first insert -> Ok(None)
    let old = doc
        .insert_fixture(fxt.clone())
        .expect("fixture insert should succeed");
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

    // second insert with same id -> Ok(Some(previous))
    let old2 = doc
        .insert_fixture(fxt)
        .expect("fixture re-insert should succeed");
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
fn insert_fixture_errors_when_fixture_def_missing() {
    let mut doc = Doc::new();

    // Universe exists
    let uni_id = UniverseId::new(1);
    doc.add_universe(uni_id);

    // Create a fixture referencing a non-existent FixtureDef
    let missing_def = FixtureDefId::new();
    let fxt = crate::fixture::Fixture::new(
        "FxMissingDef",
        uni_id,
        DmxAddress::new(1).expect("valid address"),
        missing_def,
        String::from("ModeA"),
    );

    let err = doc
        .insert_fixture(fxt)
        .err()
        .expect("expected insert error due to missing fixture def");
    assert!(matches!(
        err,
        FixtureInsertError::FixtureDefNotFound(FixtureDefNotFound { fixture_id: _, fixture_def_id })
        if fixture_def_id == missing_def
    ));
}

#[test]
fn insert_fixture_errors_when_mode_not_found_in_def() {
    let mut doc = Doc::new();

    // Prepare a def with only "ModeB"
    let def = make_fixture_def_with_mode(
        "ModelX",
        "ModeB",
        "Dimmer",
        1,
        MergeMode::HTP,
        ChannelKind::Dimmer,
    );
    let def_id = def.id();
    doc.insert_fixture_def(def);

    // Universe exists
    let uni_id = UniverseId::new(1);
    doc.add_universe(uni_id);

    // Fixture requests "ModeA" which is not present
    let fxt = make_fixture(
        "FxWrongMode",
        def_id,
        uni_id,
        DmxAddress::new(10).unwrap(),
        "ModeA",
    );

    let err = doc
        .insert_fixture(fxt)
        .err()
        .expect("expected insert error due to mode not found");
    assert!(matches!(
        err,
        FixtureInsertError::ModeNotFound (ModeNotFound{ fixture_def, mode })
        if fixture_def == def_id && mode == "ModeA"
    ));
}

#[test]
fn insert_fixture_errors_when_address_validation_fails_due_to_overlap() {
    let mut doc = Doc::new();

    // Prepare a def with "ModeA" and a single channel at offset 0 (occupies base address)
    let def = make_fixture_def_with_mode(
        "ModelOverlap",
        "ModeA",
        "Dimmer",
        0,
        MergeMode::LTP,
        ChannelKind::Dimmer,
    );
    let def_id = def.id();
    doc.insert_fixture_def(def);

    // Universe exists
    let uni_id = UniverseId::new(1);
    doc.add_universe(uni_id);

    // First fixture at address 100
    let fxt1 = make_fixture(
        "Fx1",
        def_id,
        uni_id,
        DmxAddress::new(100).unwrap(),
        "ModeA",
    );
    let fxt1_id = fxt1.id();
    let prev = doc
        .insert_fixture(fxt1)
        .expect("first fixture insert should succeed");
    assert!(prev.is_none());
    assert!(doc.get_fixture(&fxt1_id).is_some());

    // Second fixture tries to occupy the same address -> should fail validation
    let fxt2 = make_fixture(
        "Fx2",
        def_id,
        uni_id,
        DmxAddress::new(100).unwrap(),
        "ModeA",
    );
    let err = doc
        .insert_fixture(fxt2)
        .err()
        .expect("expected insert error due to address overlap");
    // Only check that this maps to AddressValidateError (inner type opaque here)
    assert!(matches!(err, FixtureInsertError::AddressValidateError(_)));
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
    let def = make_fixture_def_with_mode(
        "ModelX",
        "ModeA",
        "Dimmer",
        0,
        MergeMode::HTP,
        ChannelKind::Dimmer,
    );
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
    doc.insert_fixture(fxt).expect("should work");

    // remove once -> Ok(Some(...))
    let removed = doc
        .remove_fixture(&fxt_id)
        .expect("fixture removal should succeed");
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

    // remove again -> Ok(None)
    assert!(
        doc.remove_fixture(&fxt_id)
            .expect("second fixture removal should succeed")
            .is_none()
    );
}

#[test]
fn remove_fixture_errors_when_fixture_def_missing() {
    let mut doc = Doc::new();

    // prepare and insert fixture def + universe + fixture
    let def = make_fixture_def_with_mode(
        "ModelX",
        "ModeA",
        "Dimmer",
        0,
        MergeMode::LTP,
        ChannelKind::Dimmer,
    );
    let def_id = def.id();
    doc.insert_fixture_def(def);

    let uni_id = UniverseId::new(2);
    doc.add_universe(uni_id);

    let fxt = make_fixture(
        "FxToRemove",
        def_id,
        uni_id,
        DmxAddress::new(5).unwrap(),
        "ModeA",
    );
    let fxt_id = fxt.id();
    doc.insert_fixture(fxt)
        .expect("fixture insert should succeed");

    // Remove the fixture_def before removing fixture -> should error
    let _removed_def = doc.remove_fixture_def(&def_id);
    assert!(_removed_def.is_some());

    let err = doc
        .remove_fixture(&fxt_id)
        .err()
        .expect("expected remove_fixture to error due to missing fixture def");
    assert!(matches!(
        err,
        FixtureRemoveError::FixtureDefNotFound(FixtureDefNotFound { fixture_id, fixture_def_id })
        if fixture_id == fxt_id && fixture_def_id == def_id
    ));
}

#[test]
fn resolve_address_fails_after_fixture_removed() {
    let mut doc = Doc::new();

    // prepare dependencies: fixture def + universe
    let def = make_fixture_def_with_mode(
        "ModelX",
        "ModeA",
        "Dimmer",
        5,
        MergeMode::LTP,
        ChannelKind::Dimmer,
    );
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
    doc.insert_fixture(fxt)
        .expect("fixture insert should succeed");

    // resolve works before removal
    let (resolved_uni, resolved) = doc.resolve_address(fxt_id, "Dimmer").unwrap();
    assert_eq!(resolved_uni, uni_id);
    assert_eq!(resolved.address.value(), base + 5);

    // remove and then resolution should fail
    doc.remove_fixture(&fxt_id)
        .expect("fixture removal should succeed");
    let err = doc.resolve_address(fxt_id, "Dimmer").err().unwrap();
    assert!(matches!(err, ResolveError::FixtureNotFound(FixtureNotFound(id)) if id == fxt_id));
}
