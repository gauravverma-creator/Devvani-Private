use crate::ssd_tier::SsdTier;
use crate::types::ReversibleOp;
use tempfile::TempDir;

fn make_op(id: u64) -> ReversibleOp {
    ReversibleOp {
        id,
        forward_fn_name: "yoga".to_string(),
        inverse_fn_name: "viyoga".to_string(),
        input_snapshot: vec![id as u8],
        output_snapshot: vec![id as u8 + 1],
        ancilla_id: id,
        timestamp: id,
        is_consumed: false,
        dependencies: vec![],
    }
}

#[test]
fn test_ssd_tier_accept_and_flush() {
    let dir = TempDir::new().unwrap();
    let mut tier = SsdTier::new(dir.path(), 4).unwrap();
    for i in 0..4 {
        tier.accept(make_op(i)).unwrap();
    }
    assert_eq!(tier.coalesce_buffer_len(), 0);
}

#[test]
fn test_ssd_tier_fetch_after_flush() {
    let dir = TempDir::new().unwrap();
    let mut tier = SsdTier::new(dir.path(), 2).unwrap();
    tier.accept(make_op(0)).unwrap();
    tier.accept(make_op(1)).unwrap();
    let fetched = tier.fetch(0).unwrap();
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().id, 0);
}

#[test]
fn test_ssd_tier_fetch_nonexistent() {
    let dir = TempDir::new().unwrap();
    let tier = SsdTier::new(dir.path(), 4).unwrap();
    let result = tier.fetch(999).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_ssd_tier_manual_flush() {
    let dir = TempDir::new().unwrap();
    let mut tier = SsdTier::new(dir.path(), 100).unwrap();
    tier.accept(make_op(0)).unwrap();
    tier.accept(make_op(1)).unwrap();
    assert_eq!(tier.coalesce_buffer_len(), 2);
    tier.flush().unwrap();
    assert_eq!(tier.coalesce_buffer_len(), 0);
    let fetched = tier.fetch(1).unwrap();
    assert!(fetched.is_some());
}
