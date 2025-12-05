use std::collections::HashMap;

use super::helpers::{make_fixture, make_fixture_def_with_mode};
use crate::{
    doc::{Doc, ResolveError},
    fixture::{Fixture, FixtureId, MergeMode},
    fixture_def::{ChannelDef, FixtureDef, FixtureDefId, FixtureMode},
    universe::{DmxAddress, UniverseId},
};

#[test]
fn resolve_success_single_channel() {
    let mut doc = Doc::new();

    // Prepare a def with ModeA -> "Dimmer" at offset 7, LTP
    let def = make_fixture_def_with_mode("ModelX", "ModeA", "Dimmer", 7, MergeMode::LTP);
    let def_id = def.id();
    doc.insert_fixture_def(def);

    // Universe and fixture
    let uni_id = UniverseId::new(1);
    doc.add_universe(uni_id);
    let base_addr = 100;
    let fxt = make_fixture(
        "Fxt1",
        def_id,
        uni_id,
        DmxAddress::new(base_addr).expect("valid address"),
        "ModeA",
    );
    let fxt_id = fxt.id();
    doc.insert_fixture(fxt);

    // Resolve
    let (resolved_uni, resolved) = doc
        .resolve_address(fxt_id, "Dimmer")
        .expect("should resolve");
    assert_eq!(resolved_uni, uni_id);
    assert_eq!(resolved.address.value(), base_addr + 7);
    assert!(matches!(resolved.merge_mode, MergeMode::LTP));
}

#[test]
fn resolve_error_fixture_not_found() {
    let doc = Doc::new();

    let missing = FixtureId::new();
    let err = doc
        .resolve_address(missing, "Dimmer")
        .expect_err("should error");
    assert!(matches!(err, ResolveError::FixtureNotFound(id) if id == missing));
}

#[test]
fn resolve_error_fixture_def_not_found() {
    let mut doc = Doc::new();

    // Setup universe only
    let uni_id = UniverseId::new(1);
    doc.add_universe(uni_id);

    // Create a fixture that references a non-existent FixtureDef
    let missing_def = FixtureDefId::new();
    let fxt = Fixture::new(
        "FxtMissingDef",
        uni_id,
        DmxAddress::new(1).unwrap(),
        missing_def,
        String::from("AnyMode"),
    );
    let fxt_id = fxt.id();
    doc.insert_fixture(fxt);

    // Resolve should fail with FixtureDefNotFound
    let err = doc
        .resolve_address(fxt_id, "Dimmer")
        .expect_err("should error");
    assert!(matches!(
        err,
        ResolveError::FixtureDefNotFound {
            fixture_id,
            fixture_def_id
        } if fixture_id == fxt_id && fixture_def_id == missing_def
    ));
}

#[test]
fn resolve_error_mode_not_found() {
    let mut doc = Doc::new();

    // Def has only "ModeB", but fixture will use "ModeA"
    let def = make_fixture_def_with_mode("ModelX", "ModeB", "Dimmer", 1, MergeMode::HTP);
    let def_id = def.id();
    doc.insert_fixture_def(def);

    let uni_id = UniverseId::new(1);
    doc.add_universe(uni_id);

    let fxt = make_fixture(
        "FxtWrongMode",
        def_id,
        uni_id,
        DmxAddress::new(10).unwrap(),
        "ModeA",
    );
    let fxt_id = fxt.id();
    doc.insert_fixture(fxt);

    let err = doc
        .resolve_address(fxt_id, "Dimmer")
        .expect_err("should error");
    assert!(matches!(
        err,
        ResolveError::ModeNotFound { fixture_def, mode }
        if fixture_def == def_id && mode == "ModeA"
    ));
}

#[test]
fn resolve_error_channel_not_found_entry_present_but_none() {
    let mut doc = Doc::new();

    // Build a def with a mode "ModeA" where "Dimmer" key exists but value is None
    let mut def = FixtureDef::new(String::from("Manufacturer"), String::from("ModelX"));
    let mut order: HashMap<String, Option<(usize, ChannelDef)>> = HashMap::new();
    order.insert(String::from("Dimmer"), None);
    let mode = FixtureMode {
        channel_order: order,
    };
    def.add_mode(String::from("ModeA"), mode);
    let def_id = def.id();
    doc.insert_fixture_def(def);

    let uni_id = UniverseId::new(1);
    doc.add_universe(uni_id);

    let fxt = make_fixture(
        "FxtNoChannel",
        def_id,
        uni_id,
        DmxAddress::new(5).unwrap(),
        "ModeA",
    );
    let fxt_id = fxt.id();
    doc.insert_fixture(fxt);

    let err = doc
        .resolve_address(fxt_id, "Dimmer")
        .expect_err("should error");
    assert!(matches!(
        err,
        ResolveError::ChannelNotFound { fixturedef, mode, channel }
        if fixturedef == def_id && mode == "ModeA" && channel == "Dimmer"
    ));
}
