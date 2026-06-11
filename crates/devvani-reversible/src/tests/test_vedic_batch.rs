use crate::vedic_batch::VedicBatchEngine;
use crate::window::WindowConfig;
use tempfile::TempDir;

fn make_engine(batch_size: usize) -> (VedicBatchEngine, TempDir) {
    let dir = TempDir::new().unwrap();
    let engine = VedicBatchEngine::new(
        4 * 1024 * 1024,
        WindowConfig {
            max_ops: 100,
            purge_fraction: 0.80,
            dependency_check: true,
        },
        dir.path(),
        4,
        batch_size,
    )
    .unwrap();
    (engine, dir)
}

#[test]
fn test_record_ops_below_batch_threshold() {
    let (mut engine, _dir) = make_engine(10);
    for i in 0u8..5 {
        engine
            .record("yoga", "viyoga", vec![i], vec![i + 1], vec![])
            .unwrap();
    }
    assert_eq!(engine.batch_buffer_len(), 5);
    assert_eq!(engine.batch_stats().total_batches_processed, 0);
}

#[test]
fn test_batch_triggers_at_threshold() {
    let (mut engine, _dir) = make_engine(4);
    for i in 0u8..4 {
        engine
            .record("yoga", "viyoga", vec![i], vec![i + 1], vec![])
            .unwrap();
    }
    // Batch of 4 should have been processed
    assert_eq!(engine.batch_stats().total_batches_processed, 1);
    assert_eq!(engine.batch_buffer_len(), 0);
}

#[test]
fn test_history_reduction_achieved() {
    let (mut engine, _dir) = make_engine(8);
    for i in 0u8..8 {
        engine
            .record("yoga", "viyoga", vec![i], vec![i + 1], vec![])
            .unwrap();
    }
    let stats = engine.batch_stats();
    assert_eq!(stats.total_batches_processed, 1);
    // Anurupyena should fire (all same forward_fn) — 75% reduction
    assert!(stats.total_ops_compressed > 0);
    assert!(stats.last_reduction_fraction > 0.0);
}

#[test]
fn test_flush_batch_processes_partial() {
    let (mut engine, _dir) = make_engine(10);
    for i in 0u8..3 {
        engine
            .record("guna", "bhaga", vec![i], vec![i * 3], vec![])
            .unwrap();
    }
    assert_eq!(engine.batch_buffer_len(), 3);
    engine.flush_batch().unwrap();
    assert_eq!(engine.batch_buffer_len(), 0);
}

#[test]
fn test_cumulative_reduction_tracked() {
    let (mut engine, _dir) = make_engine(4);
    // Process 3 batches
    for _ in 0..3 {
        for i in 0u8..4 {
            engine
                .record("yoga", "viyoga", vec![i], vec![i + 1], vec![])
                .unwrap();
        }
    }
    let stats = engine.batch_stats();
    assert_eq!(stats.total_batches_processed, 3);
    assert!(stats.cumulative_reduction_fraction > 0.0);
}
