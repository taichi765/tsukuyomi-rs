use super::helpers::make_doc_handle_with_observer;
use crate::{
    doc::{DocEvent, DocStore, OutputMapError},
    engine::OutputPluginId,
    universe::UniverseId,
};

/* ==================== DocStore direct tests ==================== */

#[test]
fn add_universe_returns_none_then_some() {
    let mut doc = DocStore::new();

    let uni_id = UniverseId::new(1);

    // first add -> None
    let prev = doc.add_universe(uni_id);
    assert!(prev.is_none());
    assert!(doc.universe_settings().contains_key(&uni_id));

    // second add (same id) -> Some(old)
    let prev2 = doc.add_universe(uni_id);
    assert!(prev2.is_some());
}

#[test]
fn remove_universe_returns_some_then_none() {
    let mut doc = DocStore::new();

    let uni_id = UniverseId::new(1);

    // setup: add first
    doc.add_universe(uni_id);
    assert!(doc.universe_settings().contains_key(&uni_id));

    // first remove -> Some
    let removed = doc.remove_universe(&uni_id);
    assert!(removed.is_some());
    assert!(!doc.universe_settings().contains_key(&uni_id));

    // second remove -> None
    assert!(doc.remove_universe(&uni_id).is_none());
}

#[test]
fn add_output_inserts_once() {
    let mut doc = DocStore::new();

    let uni_id = UniverseId::new(1);
    let plugin_id = OutputPluginId::new();

    // must create universe first
    doc.add_universe(uni_id);

    // first add_output -> Ok(true)
    let added = doc
        .add_output(uni_id, plugin_id)
        .expect("add_output should succeed");
    assert!(added);

    // plugin should be present in universe setting
    let setting = doc.universe_settings().get(&uni_id).unwrap();
    assert!(setting.output_plugins().contains(&plugin_id));

    // second add_output (same plugin) -> Ok(false)
    let added_again = doc
        .add_output(uni_id, plugin_id)
        .expect("duplicate add_output should succeed");
    assert!(!added_again);

    // the plugin remains present
    let setting2 = doc.universe_settings().get(&uni_id).unwrap();
    assert!(setting2.output_plugins().contains(&plugin_id));
}

#[test]
fn remove_output_removes() {
    let mut doc = DocStore::new();

    let uni_id = UniverseId::new(1);
    let plugin_id = OutputPluginId::new();

    doc.add_universe(uni_id);
    assert!(doc.add_output(uni_id, plugin_id).unwrap());

    // remove existing -> Ok(true)
    let removed = doc.remove_output(&uni_id, &plugin_id).unwrap();
    assert!(removed);

    // plugin is no longer present
    let setting = doc.universe_settings().get(&uni_id).unwrap();
    assert!(!setting.output_plugins().contains(&plugin_id));

    // remove again -> Ok(false)
    let removed_again = doc.remove_output(&uni_id, &plugin_id).unwrap();
    assert!(!removed_again);
}

#[test]
fn output_ops_on_nonexistent_universe_returns_error() {
    let mut doc = DocStore::new();

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

/* ==================== DocHandle event notification tests ==================== */

#[test]
fn doc_handle_add_universe_emits_event() {
    let (handle, _doc_store, observer) = make_doc_handle_with_observer();

    let uni_id = UniverseId::new(1);

    // first add
    let prev = handle.add_universe(uni_id);
    assert!(prev.is_none());

    {
        let obs = observer.read().unwrap();
        assert!(
            obs.events
                .iter()
                .any(|e| matches!(e, DocEvent::UniverseAdded(id) if *id == uni_id))
        );
    }

    // second add (same id) emits event again
    let prev2 = handle.add_universe(uni_id);
    assert!(prev2.is_some());

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
fn doc_handle_remove_universe_emits_event() {
    let (handle, _doc_store, observer) = make_doc_handle_with_observer();

    let uni_id = UniverseId::new(1);

    // setup: add first
    handle.add_universe(uni_id);

    // remove
    let removed = handle.remove_universe(&uni_id);
    assert!(removed.is_some());

    {
        let obs = observer.read().unwrap();
        assert!(
            obs.events
                .iter()
                .any(|e| matches!(e, DocEvent::UniverseRemoved(id) if *id == uni_id))
        );
    }
}

#[test]
fn doc_handle_add_output_emits_universe_settings_changed() {
    let (handle, _doc_store, observer) = make_doc_handle_with_observer();

    let uni_id = UniverseId::new(1);
    let plugin_id = OutputPluginId::new();

    // must create universe first
    handle.add_universe(uni_id);

    // first add_output -> Ok(true) and event UniverseSettingsChanged
    let added = handle
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

    // second add_output (same plugin) -> Ok(false) but still emits event
    // (current implementation emits on Ok(_) regardless of inserted or not)
    let added_again = handle
        .add_output(uni_id, plugin_id)
        .expect("duplicate add_output should succeed");
    assert!(!added_again);

    {
        let obs = observer.read().unwrap();
        let changed_count = obs
            .events
            .iter()
            .filter(|e| matches!(e, DocEvent::UniverseSettingsChanged))
            .count();
        // At least 2 events (one for each add_output call that succeeded)
        assert!(changed_count >= 2);
    }
}

#[test]
fn doc_handle_remove_output_emits_universe_settings_changed() {
    let (handle, _doc_store, observer) = make_doc_handle_with_observer();

    let uni_id = UniverseId::new(1);
    let plugin_id = OutputPluginId::new();

    handle.add_universe(uni_id);
    assert!(handle.add_output(uni_id, plugin_id).unwrap());

    // Clear events to focus on remove_output
    observer.write().unwrap().events.clear();

    // remove existing -> Ok(true) and UniverseSettingsChanged event
    let removed = handle.remove_output(&uni_id, &plugin_id).unwrap();
    assert!(removed);

    {
        let obs = observer.read().unwrap();
        assert!(
            obs.events
                .iter()
                .any(|e| matches!(e, DocEvent::UniverseSettingsChanged))
        );
    }
}

#[test]
fn doc_handle_output_ops_on_nonexistent_universe_does_not_emit_event() {
    let (handle, _doc_store, observer) = make_doc_handle_with_observer();

    let uni_id = UniverseId::new(1);
    let plugin_id = OutputPluginId::new();

    // no universe added

    // add_output should error and not emit event
    let err = handle.add_output(uni_id, plugin_id);
    assert!(err.is_err());

    // remove_output should error and not emit event
    let err2 = handle.remove_output(&uni_id, &plugin_id);
    assert!(err2.is_err());

    // No UniverseSettingsChanged event should be emitted
    {
        let obs = observer.read().unwrap();
        assert!(
            !obs.events
                .iter()
                .any(|e| matches!(e, DocEvent::UniverseSettingsChanged))
        );
    }
}
