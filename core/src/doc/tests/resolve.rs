use super::helpers::{make_fixture, make_fixture_def_with_mode};
use crate::{
    doc::{DocStore, FixtureNotFound, ResolveError},
    fixture::{FixtureId, MergeMode},
    fixture_def::{ChannelKind, FixtureDef, FixtureMode},
    universe::{DmxAddress, UniverseId},
};

#[test]
fn resolve_success_single_channel() {
    let mut doc = DocStore::new();

    // Prepare a def with ModeA -> "Dimmer" at offset 7, LTP
    let def = make_fixture_def_with_mode(
        "ModelX",
        "ModeA",
        "Dimmer",
        7,
        MergeMode::LTP,
        ChannelKind::Dimmer,
    );
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
    doc.add_fixture(fxt).expect("should work");

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
    let doc = DocStore::new();

    let missing = FixtureId::new();
    let err = doc
        .resolve_address(missing, "Dimmer")
        .expect_err("should error");
    assert!(matches!(err, ResolveError::FixtureNotFound(FixtureNotFound(id)) if id == missing));
}

#[test]
fn resolve_error_channel_not_found() {
    let mut doc = DocStore::new();

    // Build a def with a mode "ModeA" that has only "Dimmer" channel
    let mut def = FixtureDef::new(String::from("Manufacturer"), String::from("ModelX"));
    let order = vec![("Dimmer".to_string(), 0)];
    let mode = FixtureMode::new(order.into_iter()).unwrap();
    def.insert_mode(String::from("ModeA"), mode);
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
    doc.add_fixture(fxt).expect("should work");

    // Try to resolve a channel that doesn't exist in the mode
    let err = doc
        .resolve_address(fxt_id, "NonExistentChannel")
        .expect_err("should error");
    assert!(matches!(
        err,
        ResolveError::ChannelNotFound { fixture_def, mode, channel }
        if fixture_def == def_id && mode == "ModeA" && channel == "NonExistentChannel"
    ));
}
