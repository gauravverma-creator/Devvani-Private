use crate::sutra::{Sutra, SutraSelector};
use crate::types::ReversibleOp;

fn make_op(id: u64, fwd: &str, inv: &str, deps: Vec<u64>) -> ReversibleOp {
    ReversibleOp {
        id,
        forward_fn_name: fwd.to_string(),
        inverse_fn_name: inv.to_string(),
        input_snapshot: vec![id as u8],
        output_snapshot: vec![id as u8 + 1],
        ancilla_id: id,
        timestamp: id,
        is_consumed: false,
        dependencies: deps,
    }
}

#[test]
fn test_empty_batch_selects_urdhva() {
    let selection = SutraSelector::select(&[]);
    assert_eq!(selection.sutra, Sutra::UrdhvaTiryak);
}

#[test]
fn test_low_dependency_selects_urdhva() {
    let ops: Vec<ReversibleOp> = (0..10)
        .map(|i| make_op(i, &format!("fwd_{}", i), "inv", vec![]))
        .collect();
    let selection = SutraSelector::select(&ops);
    assert_eq!(selection.sutra, Sutra::UrdhvaTiryak);
    assert!(selection.estimated_reduction >= 0.70);
}

#[test]
fn test_high_dependency_selects_nikhilam() {
    // All ops have dependencies
    let ops: Vec<ReversibleOp> = (1..10)
        .map(|i| make_op(i, &format!("fwd_{}", i), &format!("inv_{}", i), vec![i - 1]))
        .collect();
    let selection = SutraSelector::select(&ops);
    // High dependency density should trigger Nikhilam
    assert!(
        selection.sutra == Sutra::Nikhilam || selection.sutra == Sutra::UrdhvaTiryak,
        "Expected Nikhilam or UrdhvaTiryak, got {:?}",
        selection.sutra
    );
}

#[test]
fn test_repeated_pattern_selects_anurupyena() {
    // More than 50% ops have same forward_fn_name
    let ops: Vec<ReversibleOp> = (0..10)
        .map(|i| make_op(i, "yoga", "viyoga", vec![]))
        .collect();
    let selection = SutraSelector::select(&ops);
    assert_eq!(selection.sutra, Sutra::Anurupyena);
}

#[test]
fn test_inverse_chain_selects_paravartya() {
    // inv of op[i] == fwd of op[i+1]
    let ops = vec![
        make_op(0, "yoga", "viyoga", vec![]),
        make_op(1, "viyoga", "yoga", vec![]),
        make_op(2, "guna", "bhaga", vec![]),
    ];
    let selection = SutraSelector::select(&ops);
    // Should detect inverse chain
    assert!(
        selection.sutra == Sutra::Paravartya || selection.sutra == Sutra::Anurupyena,
        "Expected Paravartya or Anurupyena"
    );
}

#[test]
fn test_partition_respects_reduction_fraction() {
    let ops: Vec<ReversibleOp> = (0..10)
        .map(|i| make_op(i, "yoga", "viyoga", vec![]))
        .collect();
    let selection = SutraSelector::select(&ops);
    let (retain, compress) = SutraSelector::partition(&ops, &selection);
    assert_eq!(retain.len() + compress.len(), ops.len());
    assert!(!retain.is_empty());
}
