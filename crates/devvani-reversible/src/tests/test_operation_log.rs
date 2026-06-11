use crate::operation_log::OperationLog;
use crate::types::{ReversibleOp, SideEffectMarker, SideEffectType, UncomputeResult};

fn make_op(id: u64) -> ReversibleOp {
    ReversibleOp {
        id,
        forward_fn_name: "yoga".to_string(),
        inverse_fn_name: "viyoga".to_string(),
        input_snapshot: vec![1],
        output_snapshot: vec![2],
        ancilla_id: id,
        timestamp: id,
        is_consumed: false,
        dependencies: vec![],
    }
}

#[test]
fn test_append_and_forward_chain() {
    let mut log = OperationLog::new();
    log.append(make_op(0));
    log.append(make_op(1));
    log.append(make_op(2));
    assert_eq!(log.forward_chain(), &[0, 1, 2]);
    assert_eq!(log.total_ops(), 3);
}

#[test]
fn test_uncompute_chain_is_reverse() {
    let mut log = OperationLog::new();
    log.append(make_op(0));
    log.append(make_op(1));
    log.append(make_op(2));
    let chain = log.uncompute_chain();
    assert_eq!(chain, &[2, 1, 0]);
}

#[test]
fn test_get_op_by_id() {
    let mut log = OperationLog::new();
    log.append(make_op(42));
    let op = log.get_op(42).unwrap();
    assert_eq!(op.id, 42);
    assert_eq!(op.forward_fn_name, "yoga");
}

#[test]
fn test_side_effect_recording() {
    let mut log = OperationLog::new();
    log.record_side_effect(SideEffectMarker {
        op_id: 0,
        effect_type: SideEffectType::IoWrite,
        description: "vadati called".to_string(),
    });
    assert_eq!(log.total_side_effects(), 1);
    assert_eq!(log.side_effects()[0].op_id, 0);
}

#[test]
fn test_uncompute_result_recording() {
    let mut log = OperationLog::new();
    log.record_uncompute_result(UncomputeResult::Success {
        op_id: 0,
        ancilla_id: 0,
    });
    log.record_uncompute_result(UncomputeResult::AlreadyConsumed { op_id: 1 });
    assert_eq!(log.uncompute_results().len(), 2);
}