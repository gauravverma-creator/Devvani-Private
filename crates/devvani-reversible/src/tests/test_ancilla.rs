use crate::ancilla::AncillaStore;
use crate::types::ReversibleOp;

fn make_op(id: u64, deps: Vec<u64>) -> ReversibleOp {
    ReversibleOp {
        id,
        forward_fn_name: "yoga".to_string(),
        inverse_fn_name: "viyoga".to_string(),
        input_snapshot: vec![1, 2, 3],
        output_snapshot: vec![4, 5, 6],
        ancilla_id: 0,
        timestamp: id,
        is_consumed: false,
        dependencies: deps,
    }
}

#[test]
fn test_store_and_retrieve() {
    let mut store = AncillaStore::new();
    let op = make_op(0, vec![]);
    let ancilla_id = store.store(op.clone());
    let retrieved = store.get(ancilla_id).unwrap();
    assert_eq!(retrieved.id, 0);
    assert_eq!(retrieved.forward_fn_name, "yoga");
}

#[test]
fn test_mark_consumed() {
    let mut store = AncillaStore::new();
    let op = make_op(0, vec![]);
    let ancilla_id = store.store(op);
    assert!(store.mark_consumed(ancilla_id).is_ok());
    let op = store.get(ancilla_id).unwrap();
    assert!(op.is_consumed);
}

#[test]
fn test_double_consume_returns_error() {
    let mut store = AncillaStore::new();
    let op = make_op(0, vec![]);
    let ancilla_id = store.store(op);
    store.mark_consumed(ancilla_id).unwrap();
    let result = store.mark_consumed(ancilla_id);
    assert!(result.is_err());
}

#[test]
fn test_active_count() {
    let mut store = AncillaStore::new();
    store.store(make_op(0, vec![]));
    store.store(make_op(1, vec![]));
    store.store(make_op(2, vec![]));
    assert_eq!(store.active_count(), 3);
    let id = store.store(make_op(3, vec![]));
    // consume ancilla_id = id (which is 3, since store assigns sequentially)
    store.mark_consumed(id).unwrap();
    assert_eq!(store.active_count(), 3);
}

#[test]
fn test_purge_consumed_no_dependents() {
    let mut store = AncillaStore::new();
    let a0 = store.store(make_op(0, vec![]));
    let a1 = store.store(make_op(1, vec![]));
    store.mark_consumed(a0).unwrap();
    store.mark_consumed(a1).unwrap();
    let purged = store.purge_consumed();
    assert_eq!(purged, 2);
    assert_eq!(store.active_count(), 0);
}

#[test]
fn test_purge_respects_dependency() {
    let mut store = AncillaStore::new();
    // op0 is a dependency of op1
    let a0 = store.store(make_op(0, vec![]));
    // op1 depends on op0's ancilla_id (which is 0)
    let _a1 = store.store(make_op(1, vec![a0]));
    store.mark_consumed(a0).unwrap();
    // op1 is still active, so op0's ancilla should NOT be purged
    let purged = store.purge_consumed();
    assert_eq!(purged, 0);
}