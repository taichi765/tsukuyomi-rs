use std::sync::{Arc, RwLock};

use super::helpers::{TestObserver, make_function};
use crate::doc::{Doc, DocEvent, DocObserver};

#[test]
fn add_and_remove_function_emits_events_and_updates_store() {
    let mut doc = Doc::new();

    // subscribe observer
    let observer = Arc::new(RwLock::new(TestObserver::new()));
    {
        let obs: Arc<RwLock<dyn DocObserver>> = Arc::clone(&observer) as _;
        doc.subscribe(Arc::downgrade(&obs));
    }

    // create a function
    let func = make_function("Func1");
    let func_id = func.id();

    // add -> None
    let old = doc.add_function(func);
    assert!(old.is_none());

    // get_function_data should return Some
    let got = doc.get_function_data(func_id);
    assert!(got.is_some());
    assert_eq!(got.unwrap().id(), func_id);

    // event emitted for insertion
    {
        let obs = observer.read().unwrap();
        assert!(
            obs.events
                .iter()
                .any(|e| matches!(e, DocEvent::FunctionInserted(id) if *id == func_id))
        );
    }

    // remove -> Some
    let removed = doc.remove_function(&func_id);
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().id(), func_id);

    // get_function_data should return None
    assert!(doc.get_function_data(&func_id).is_none());

    // event emitted for removal
    {
        let obs = observer.read().unwrap();
        assert!(
            obs.events
                .iter()
                .any(|e| matches!(e, DocEvent::FunctionRemoved(id) if *id == func_id))
        );
    }
}

#[test]
fn remove_nonexistent_function_returns_none() {
    let mut doc = Doc::new();

    let func = make_function("NonExistent");
    let func_id = func.id();
    // do not insert

    // Removing a never inserted function should be None
    assert!(doc.remove_function(&func_id).is_none());
}

#[test]
fn add_multiple_functions_and_remove_one_keeps_the_other() {
    let mut doc = Doc::new();

    // subscribe observer
    let observer = Arc::new(RwLock::new(TestObserver::new()));
    {
        let obs: Arc<RwLock<dyn DocObserver>> = Arc::clone(&observer) as _;
        doc.subscribe(Arc::downgrade(&obs));
    }

    // create two functions
    let func1 = make_function("Func1");
    let func2 = make_function("Func2");
    let func1_id = func1.id();
    let func2_id = func2.id();

    // add both
    assert!(doc.add_function(func1).is_none());
    assert!(doc.add_function(func2).is_none());

    // both should exist
    assert!(doc.get_function_data(&func1_id).is_some());
    assert!(doc.get_function_data(&func2_id).is_some());

    // remove one
    let removed = doc.remove_function(&func1_id);
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().id(), func1_id);

    // one removed, the other remains
    assert!(doc.get_function_data(&func1_id).is_none());
    assert!(doc.get_function_data(&func2_id).is_some());

    // events sanity check (at least inserted for both and removed for id1)
    {
        let obs = observer.read().unwrap();
        let inserted_1 = obs
            .events
            .iter()
            .filter(|e| matches!(e, DocEvent::FunctionInserted(id) if *id == func1_id))
            .count();
        let inserted_2 = obs
            .events
            .iter()
            .filter(|e| matches!(e, DocEvent::FunctionInserted(id) if *id == func2_id))
            .count();
        let removed_1 = obs
            .events
            .iter()
            .filter(|e| matches!(e, DocEvent::FunctionRemoved(id) if *id == func1_id))
            .count();

        assert!(inserted_1 >= 1);
        assert!(inserted_2 >= 1);
        assert!(removed_1 >= 1);
    }
}
