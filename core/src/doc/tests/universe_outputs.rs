use std::sync::{Arc, RwLock};

use super::helpers::TestObserver;
use crate::{
    doc::{Doc, DocEvent, DocObserver, OutputMapError},
    engine::OutputPluginId,
    universe::UniverseId,
};

#[test]
fn add_universe_returns_none_then_some_and_emits_events() {
    let mut doc = Doc::new();

    let observer = Arc::new(RwLock::new(TestObserver::new()));
    {
        let obs: Arc<RwLock<dyn DocObserver>> = Arc::clone(&observer) as _;
        doc.subscribe(Arc::downgrade(&obs));
    }

    let uni_id = UniverseId::new(1);

    // first add -> None
    let prev = doc.add_universe(uni_id);
    assert!(prev.is_none());
    assert!(doc.universe_settings().contains_key(&uni_id));

    {
        let obs = observer.read().unwrap();
        assert!(
            obs.events
                .iter()
                .any(|e| matches!(e, DocEvent::UniverseAdded(id) if *id == uni_id))
        );
    }

    // second add (same id) -> Some(old)
    let prev2 = doc.add_universe(uni_id);
    assert!(prev2.is_some());

    // another UniverseAdded event should be emitted
    {
        let obs = observer.read().unwrap();
        let count = obs
            .events
            .iter()
            .filter(|e| matches!(e, DocEvent::UniverseAdded(id) if *id == uni_id))
            .count();
        assert!(count >= 2);
    }
}

#[test]
fn remove_universe_returns_some_then_none_and_emits_event() {
    let mut doc = Doc::new();

    let observer = Arc::new(RwLock::new(TestObserver::new()));
    {
        let obs: Arc<RwLock<dyn DocObserver>> = Arc::clone(&observer) as _;
        doc.subscribe(Arc::downgrade(&obs));
    }

    let uni_id = UniverseId::new(1);

    // setup: add first
    doc.add_universe(uni_id);
    assert!(doc.universe_settings().contains_key(&uni_id));

    // first remove -> Some
    let removed = doc.remove_universe(&uni_id);
    assert!(removed.is_some());
    assert!(!doc.universe_settings().contains_key(&uni_id));

    {
        let obs = observer.read().unwrap();
        assert!(
            obs.events
                .iter()
                .any(|e| matches!(e, DocEvent::UniverseRemoved(id) if *id == uni_id))
        );
    }

    // second remove -> None
    assert!(doc.remove_universe(&uni_id).is_none());
}

#[test]
fn add_output_inserts_once_and_emits_universe_settings_changed() {
    let mut doc = Doc::new();

    let observer = Arc::new(RwLock::new(TestObserver::new()));
    {
        let obs: Arc<RwLock<dyn DocObserver>> = Arc::clone(&observer) as _;
        doc.subscribe(Arc::downgrade(&obs));
    }

    let uni_id = UniverseId::new(1);
    let plugin_id = OutputPluginId::new();

    // must create universe first
    doc.add_universe(uni_id);

    // first add_output -> Ok(true) and event UniverseSettingsChanged
    let added = doc
        .add_output(uni_id, plugin_id)
        .expect("add_output should succeed");
    assert!(added);

    {
        let obs = observer.read().unwrap();
        assert!(
            obs.events
                .iter()
                .any(|e| matches!(e, DocEvent::UniverseSettingsChanged))
        );
    }

    // plugin should be present in universe setting
    let setting = doc.universe_settings().get(&uni_id).unwrap();
    assert!(setting.output_plugins().contains(&plugin_id));

    // second add_output (same plugin) -> Ok(false) and typically no extra settings-changed event
    let added_again = doc
        .add_output(uni_id, plugin_id)
        .expect("duplicate add_output should succeed");
    assert!(!added_again);

    // the plugin remains present
    let setting2 = doc.universe_settings().get(&uni_id).unwrap();
    assert!(setting2.output_plugins().contains(&plugin_id));

    // optional: ensure we didn't emit a second UniverseSettingsChanged on duplicate insert
    {
        let obs = observer.read().unwrap();
        let changed_count = obs
            .events
            .iter()
            .filter(|e| matches!(e, DocEvent::UniverseSettingsChanged))
            .count();
        assert_eq!(changed_count, 1);
    }
}

#[test]
fn remove_output_removes_and_emits_universe_removed_event() {
    let mut doc = Doc::new();

    let observer = Arc::new(RwLock::new(TestObserver::new()));
    {
        let obs: Arc<RwLock<dyn DocObserver>> = Arc::clone(&observer) as _;
        doc.subscribe(Arc::downgrade(&obs));
    }

    let uni_id = UniverseId::new(1);
    let plugin_id = OutputPluginId::new();

    doc.add_universe(uni_id);
    assert!(doc.add_output(uni_id, plugin_id).unwrap());

    // remove existing -> Ok(true) and UniverseRemoved event (current behavior)
    let removed = doc.remove_output(&uni_id, &plugin_id).unwrap();
    assert!(removed);

    // plugin is no longer present
    let setting = doc.universe_settings().get(&uni_id).unwrap();
    assert!(!setting.output_plugins().contains(&plugin_id));

    {
        let obs = observer.read().unwrap();
        assert!(
            obs.events
                .iter()
                .any(|e| matches!(e, DocEvent::UniverseRemoved(id) if *id == uni_id))
        );
    }

    // remove again -> Ok(false)
    let removed_again = doc.remove_output(&uni_id, &plugin_id).unwrap();
    assert!(!removed_again);
}

#[test]
fn output_ops_on_nonexistent_universe_returns_error() {
    let mut doc = Doc::new();

    let uni_id = UniverseId::new(1);
    let plugin_id = OutputPluginId::new();

    // no universe added

    // add_output should error
    let err = doc
        .add_output(uni_id, plugin_id)
        .err()
        .expect("expected add_output to error due to missing universe");
    assert!(matches!(err, OutputMapError::UniverseNotFound(id) if id==uni_id));

    // remove_output should error
    let err2 = doc
        .remove_output(&uni_id, &plugin_id)
        .err()
        .expect("expected remove_output to error due to missing universe");
    assert!(matches!(err2, OutputMapError::UniverseNotFound(id) if id==uni_id));
}
