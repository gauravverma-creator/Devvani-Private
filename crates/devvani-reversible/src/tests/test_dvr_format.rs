use crate::dvr_format::{read_dvr, write_dvr};
use crate::types::ReversibleOp;
use std::io::Cursor;

fn make_op(id: u64) -> ReversibleOp {
    ReversibleOp {
        id,
        forward_fn_name: "yoga".to_string(),
        inverse_fn_name: "viyoga".to_string(),
        input_snapshot: vec![1, 2, 3],
        output_snapshot: vec![4, 5, 6],
        ancilla_id: id,
        timestamp: id,
        is_consumed: false,
        dependencies: vec![],
    }
}

#[test]
fn test_write_read_roundtrip_empty() {
    let ops: Vec<ReversibleOp> = vec![];
    let mut buf = Cursor::new(Vec::new());
    write_dvr(&mut buf, &ops).unwrap();
    buf.set_position(0);
    let result = read_dvr(&mut buf).unwrap();
    assert_eq!(result.len(), 0);
}

#[test]
fn test_write_read_roundtrip_single() {
    let ops = vec![make_op(0)];
    let mut buf = Cursor::new(Vec::new());
    write_dvr(&mut buf, &ops).unwrap();
    buf.set_position(0);
    let result = read_dvr(&mut buf).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, 0);
    assert_eq!(result[0].forward_fn_name, "yoga");
}

#[test]
fn test_write_read_roundtrip_multiple() {
    let ops: Vec<ReversibleOp> = (0..5).map(make_op).collect();
    let mut buf = Cursor::new(Vec::new());
    write_dvr(&mut buf, &ops).unwrap();
    buf.set_position(0);
    let result = read_dvr(&mut buf).unwrap();
    assert_eq!(result.len(), 5);
    for (i, op) in result.iter().enumerate() {
        assert_eq!(op.id, i as u64);
    }
}

#[test]
fn test_invalid_magic_returns_error() {
    let bad_data = vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let mut buf = Cursor::new(bad_data);
    let result = read_dvr(&mut buf);
    assert!(result.is_err());
}
