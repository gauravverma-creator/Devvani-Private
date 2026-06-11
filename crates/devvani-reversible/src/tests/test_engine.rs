use crate::engine::ReversibleEngine;
use crate::types::{SideEffectMarker, SideEffectType, UncomputeResult};
use crate::window::WindowConfig;
use tempfile::TempDir;

fn make_engine() -> (ReversibleEngine, TempDir) {
    let dir = TempDir::new().unwrap();
    let engine = ReversibleEngine::new(
        1024 * 1024,
        WindowConfig {
            max_ops: 20,
            purge_fraction: 0.80,
            dependency_check: true,
        },
        dir.path(),
        4,
    )
    .unwrap();
    (engine, dir)
}

#[test]
fn test_engine_record_single() {
    let (mut engine, _dir) = make_engine();
    let (op_id, ancilla_id) = engine
        .record("yoga", "viyoga", vec![1, 2], vec![3, 4], vec![])
        .unwrap();
    assert_eq!(op_id, 0);
    assert_eq!(ancilla_id, 0);
}

#[test]
fn test_engine_record_and_uncompute() {
    let (mut engine, _dir) = make_engine();
    let (op_id, ancilla_id) = engine
        .record("guna", "bhaga", vec![3], vec![9], vec![])
        .unwrap();
    let result = engine.uncompute(op_id).unwrap();
    assert_eq!(result, UncomputeResult::Success { op_id, ancilla_id });
}

#[test]
fn test_engine_uncompute_all() {
    let (mut engine, _dir) = make_engine();
    for i in 0u8..5 {
        engine
            .record("yoga", "viyoga", vec![i], vec![i + 1], vec![])
            .unwrap();
    }
    let results = engine.uncompute_all().unwrap();
    assert_eq!(results.len(), 5);
    assert_eq!(engine.log().forward_chain(), &[0, 1, 2, 3, 4]);
}

#[test]
fn test_engine_side_effect_recording() {
    let (mut engine, _dir) = make_engine();
    engine.record_side_effect(SideEffectMarker {
        op_id: 0,
        effect_type: SideEffectType::IoWrite,
        description: "vadati output".to_string(),
    });
    assert_eq!(engine.log().total_side_effects(), 1);
}

#[test]
fn test_engine_stats() {
    let (mut engine, _dir) = make_engine();
    engine
        .record("yoga", "viyoga", vec![1], vec![2], vec![])
        .unwrap();
    engine
        .record("guna", "bhaga", vec![3], vec![9], vec![])
        .unwrap();
    let stats = engine.stats();
    assert_eq!(stats.total_ops_recorded, 2);
}

#[test]
fn test_engine_flush() {
    let (mut engine, _dir) = make_engine();
    engine
        .record("yoga", "viyoga", vec![1], vec![2], vec![])
        .unwrap();
    assert!(engine.flush().is_ok());
}
