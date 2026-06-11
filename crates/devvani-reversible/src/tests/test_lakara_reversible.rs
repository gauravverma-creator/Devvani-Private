use crate::lakara_reversible::{LakaaraReversible, ReversibleDiagnostic};

#[test]
fn test_pratyavartya_lit_is_reversible() {
    let l = LakaaraReversible::PratyavartyaLit {
        inverse_dhatu: "viyoga".to_string(),
    };
    assert!(l.is_reversible());
    assert_eq!(l.inverse_dhatu(), Some("viyoga"));
    assert_eq!(l.ascii_name(), "PratyavartyaLit");
}

#[test]
fn test_anapravartya_lot_is_not_reversible() {
    let l = LakaaraReversible::AnapravartyaLot {
        effect_description: "vadati call".to_string(),
    };
    assert!(!l.is_reversible());
    assert_eq!(l.inverse_dhatu(), None);
    assert_eq!(l.ascii_name(), "AnapravartyaLot");
}

#[test]
fn test_pratyavartya_lan_with_op_id() {
    let l = LakaaraReversible::PratyavartyaLan {
        inverse_dhatu: "viyoga".to_string(),
        recorded_op_id: Some(42),
    };
    assert!(l.is_reversible());
    assert_eq!(l.inverse_dhatu(), Some("viyoga"));
}

#[test]
fn test_diagnostic_codes() {
    let d = ReversibleDiagnostic::InverseDhatuNotFound {
        dhatu_name: "viyoga".to_string(),
    };
    assert_eq!(d.code(), "D020");
    assert!(d.message().contains("D020"));

    let d2 = ReversibleDiagnostic::UncomputeOnIrreversible {
        dhatu_name: "vadati".to_string(),
    };
    assert_eq!(d2.code(), "D023");
    assert!(d2.message().contains("D023"));
}

#[test]
fn test_sanskrit_names_non_empty() {
    let variants = vec![
        LakaaraReversible::PratyavartyaLit { inverse_dhatu: "x".to_string() },
        LakaaraReversible::PratyavartyaLan { inverse_dhatu: "x".to_string(), recorded_op_id: None },
        LakaaraReversible::PratyavartyaLrt { inverse_dhatu: "x".to_string() },
        LakaaraReversible::AnapravartyaLot { effect_description: "x".to_string() },
    ];
    for v in &variants {
        assert!(!v.sanskrit_name().is_empty());
        assert!(!v.ascii_name().is_empty());
    }
}
