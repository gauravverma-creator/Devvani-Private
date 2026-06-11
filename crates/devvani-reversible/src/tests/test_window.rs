use crate::window::{ComputeWindow, WindowConfig};

#[test]
fn test_window_push_no_purge() {
    let config = WindowConfig {
        max_ops: 10,
        purge_fraction: 0.80,
        dependency_check: true,
    };
    let mut window = ComputeWindow::new(config).unwrap();
    for i in 0..5 {
        let purged = window.push(i);
        assert!(purged.is_empty());
    }
    assert_eq!(window.current_size(), 5);
}

#[test]
fn test_window_triggers_purge_at_capacity() {
    let config = WindowConfig {
        max_ops: 5,
        purge_fraction: 0.80,
        dependency_check: true,
    };
    let mut window = ComputeWindow::new(config).unwrap();
    let mut all_purged = vec![];
    for i in 0..5u64 {
        let purged = window.push(i);
        all_purged.extend(purged);
    }
    // At op 4 (index 4), window hits max_ops=5, purge fires
    assert!(!all_purged.is_empty());
    // 80% of 5 = 4 ops purged
    assert_eq!(all_purged.len(), 4);
}

#[test]
fn test_window_purge_count() {
    let config = WindowConfig {
        max_ops: 10,
        purge_fraction: 0.80,
        dependency_check: true,
    };
    let window = ComputeWindow::new(config).unwrap();
    assert_eq!(window.config().purge_count(), 8);
}

#[test]
fn test_invalid_config_zero_max_ops() {
    let config = WindowConfig {
        max_ops: 0,
        purge_fraction: 0.80,
        dependency_check: true,
    };
    assert!(ComputeWindow::new(config).is_err());
}

#[test]
fn test_invalid_config_purge_fraction() {
    let config = WindowConfig {
        max_ops: 10,
        purge_fraction: 1.5,
        dependency_check: true,
    };
    assert!(ComputeWindow::new(config).is_err());
}