use crate::ram_buffer::RamBuffer;
use crate::types::UncomputeResult;
use crate::window::WindowConfig;

fn small_buffer() -> RamBuffer {
    RamBuffer::new(
        1024 * 1024, // 1MB
        WindowConfig {
            max_ops: 10,
            purge_fraction: 0.80,
            dependency_check: true,
        },
    )
    .unwrap()
}

#[test]
fn test_record_single_op() {
    let mut buf = small_buffer();
    let (op_id, ancilla_id, purged) = buf
        .record(
            "yoga".to_string(),
            "viyoga".to_string(),
            vec![1, 2, 3],
            vec![4, 5, 6],
            vec![],
        )
        .unwrap();
    assert_eq!(op_id, 0);
    assert_eq!(ancilla_id, 0);
    assert!(purged.is_empty());
}

#[test]
fn test_record_multiple_ops_sequential_ids() {
    let mut buf = small_buffer();
    for i in 0..5u64 {
        let (op_id, _, _) = buf
            .record(
                "yoga".to_string(),
                "viyoga".to_string(),
                vec![i as u8],
                vec![i as u8 + 1],
                vec![],
            )
            .unwrap();
        assert_eq!(op_id, i);
    }
}

#[test]
fn test_uncompute_success() {
    let mut buf = small_buffer();
    let (op_id, ancilla_id, _) = buf
        .record(
            "guna".to_string(),
            "bhaga".to_string(),
            vec![2],
            vec![6],
            vec![],
        )
        .unwrap();
    let result = buf.uncompute(op_id).unwrap();
    assert_eq!(result, UncomputeResult::Success { op_id, ancilla_id });
}

#[test]
fn test_uncompute_already_consumed() {
    let mut buf = small_buffer();
    let (op_id, _, _) = buf
        .record(
            "yoga".to_string(),
            "viyoga".to_string(),
            vec![1],
            vec![2],
            vec![],
        )
        .unwrap();
    buf.uncompute(op_id).unwrap();
    let result = buf.uncompute(op_id).unwrap();
    assert_eq!(result, UncomputeResult::AlreadyConsumed { op_id });
}

#[test]
fn test_stats_after_operations() {
    let mut buf = small_buffer();
    buf.record(
        "yoga".to_string(),
        "viyoga".to_string(),
        vec![1],
        vec![2],
        vec![],
    )
    .unwrap();
    buf.record(
        "guna".to_string(),
        "bhaga".to_string(),
        vec![3],
        vec![9],
        vec![],
    )
    .unwrap();
    let stats = buf.stats();
    assert_eq!(stats.total_ops_recorded, 2);
    assert_eq!(stats.total_ops_uncomputed, 0);
}

#[test]
fn test_window_purge_triggers_on_full() {
    let mut buf = small_buffer(); // max_ops = 10
    for i in 0..10 {
        buf.record(
            "yoga".to_string(),
            "viyoga".to_string(),
            vec![i as u8],
            vec![i as u8],
            vec![],
        )
        .unwrap();
    }
    // After 10 ops, window should have triggered purge
    let stats = buf.stats();
    assert_eq!(stats.total_ops_recorded, 10);
    // Window purged 80% = 8 ops, so current_window_size should be 2
    assert!(stats.current_window_size <= 2);
}
