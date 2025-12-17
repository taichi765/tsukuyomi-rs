use super::helpers::{make_doc_handle_with_observer, make_function};
use crate::doc::{DocEvent, DocStore};

/* ==================== DocStore direct tests ==================== */

#[test]
fn add_and_remove_function_updates_store() {
    let mut doc = DocStore::new();

    // create a function
    let func = make_function("Func1");
    let func_id = func.id();

    // add -> None
    let old = doc.add_function(func);
    assert!(old.is_none());

    // get_function_data should return Some
    let got = doc.get_function_data(&func_id);
    assert!(got.is_some());
    assert_eq!(got.unwrap().id(), func_id);

    // remove -> Some
    let removed = doc.remove_function(&func_id);
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().id(), func_id);

    // get_function_data should return None
    assert!(doc.get_function_data(&func_id).is_none());
}

#[test]
fn remove_nonexistent_function_returns_none() {
    let mut doc = DocStore::new();

    let func = make_function("NonExistent");
    let func_id = func.id();
    // do not insert

    // Removing a never inserted function should be None
    assert!(doc.remove_function(&func_id).is_none());
}

#[test]
fn add_multiple_functions_and_remove_one_keeps_the_other() {
    let mut doc = DocStore::new();

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
}

/* ==================== DocHandle event notification tests ==================== */

#[test]
fn doc_handle_add_function_emits_event() {
    let (handle, _doc_store, observer) = make_doc_handle_with_observer();

    // create a function
    let func = make_function("Func1");
    let func_id = func.id();

    // add function
    let old = handle.add_function(func);
    assert!(old.is_none());

    // event emitted for insertion
    {
        let obs = observer.read().unwrap();
        assert!(
            obs.events
                .iter()
                .any(|e| matches!(e, DocEvent::FunctionAdded(id) if *id == func_id))
        );
    }
}

#[test]
fn doc_handle_remove_function_emits_event() {
    let (handle, _doc_store, observer) = make_doc_handle_with_observer();

    // create and add a function
    let func = make_function("Func1");
    let func_id = func.id();
    handle.add_function(func);

    // remove function
    let removed = handle.remove_function(&func_id);
    assert!(removed.is_some());

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
fn doc_handle_multiple_function_operations_emit_correct_events() {
    let (handle, _doc_store, observer) = make_doc_handle_with_observer();

    // create two functions
    let func1 = make_function("Func1");
    let func2 = make_function("Func2");
    let func1_id = func1.id();
    let func2_id = func2.id();

    // add both
    assert!(handle.add_function(func1).is_none());
    assert!(handle.add_function(func2).is_none());

    // remove one
    let removed = handle.remove_function(&func1_id);
    assert!(removed.is_some());

    // events sanity check
    {
        let obs = observer.read().unwrap();
        let inserted_1 = obs
            .events
            .iter()
            .filter(|e| matches!(e, DocEvent::FunctionAdded(id) if *id == func1_id))
            .count();
        let inserted_2 = obs
            .events
            .iter()
            .filter(|e| matches!(e, DocEvent::FunctionAdded(id) if *id == func2_id))
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
