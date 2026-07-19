use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum DvnValue {
    Sankhya(i64),                     // integer
    Dasha(f64),                       // float
    Vakya(String),                    // string
    Satya(bool),                      // boolean
    Suchi(Vec<DvnValue>),             // list
    Sutram(Vec<DvnValue>),            // tuple (immutable)
    Kosha(HashMap<String, DvnValue>), // dict
    Samuha(HashSet<String>),          // set
    Kriya(String),                    // function reference (by name)
    Shunya,                           // None/null
}

#[derive(Debug, Error)]
pub enum StdlibError {
    #[error("Wrong number of args for {dhatu}: expected {expected}, got {got}")]
    ArgCount {
        dhatu: String,
        expected: usize,
        got: usize,
    },

    #[error("Wrong type for {dhatu}: expected {expected}, got {got}")]
    TypeError {
        dhatu: String,
        expected: String,
        got: String,
    },

    #[error("IO error in {dhatu}: {msg}")]
    IoError { dhatu: String, msg: String },

    #[error("Conversion error in {dhatu}: {msg}")]
    ConversionError { dhatu: String, msg: String },
}

pub trait DhatuFn: Send + Sync {
    fn name(&self) -> &'static str;
    fn sanskrit_root(&self) -> &'static str;
    fn python_equivalent(&self) -> &'static str;
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError>;
}

pub mod dhatu;
pub mod prelude;
pub mod registry;
pub mod string;

pub use string::{
    __devvani_vaak_khanda, __devvani_vaak_mukta, __devvani_vaak_parimana, __devvani_vaak_yoga,
};

#[cfg(test)]
mod tests {
    use super::*;
    use registry::DhatuRegistry;

    #[test]
    fn test_registry_has_70_dhatus() {
        let reg = DhatuRegistry::new();
        assert_eq!(reg.list_all().len(), 70);
    }

    #[test]
    fn test_ganavati_len() {
        let reg = DhatuRegistry::new();
        let result = reg
            .call(
                "ganavati",
                vec![DvnValue::Suchi(vec![
                    DvnValue::Sankhya(1),
                    DvnValue::Sankhya(2),
                    DvnValue::Sankhya(3),
                ])],
            )
            .unwrap();
        assert_eq!(result, DvnValue::Sankhya(3));
    }

    #[test]
    fn test_parinameti_int_conversion() {
        let reg = DhatuRegistry::new();
        let result = reg
            .call("parinameti", vec![DvnValue::Vakya("42".to_string())])
            .unwrap();
        assert_eq!(result, DvnValue::Sankhya(42));
    }

    #[test]
    fn test_yojayati_sum() {
        let reg = DhatuRegistry::new();
        let result = reg
            .call(
                "yojayati",
                vec![DvnValue::Suchi(vec![
                    DvnValue::Sankhya(10),
                    DvnValue::Sankhya(20),
                    DvnValue::Sankhya(30),
                ])],
            )
            .unwrap();
        assert_eq!(result, DvnValue::Sankhya(60));
    }

    #[test]
    fn test_kramate_range() {
        let reg = DhatuRegistry::new();
        let result = reg
            .call(
                "kramate",
                vec![
                    DvnValue::Sankhya(0),
                    DvnValue::Sankhya(3),
                    DvnValue::Sankhya(1),
                ],
            )
            .unwrap();
        assert_eq!(
            result,
            DvnValue::Suchi(vec![
                DvnValue::Sankhya(0),
                DvnValue::Sankhya(1),
                DvnValue::Sankhya(2),
            ])
        );
    }

    #[test]
    fn test_asti_isinstance() {
        let reg = DhatuRegistry::new();
        let result = reg
            .call(
                "asti",
                vec![
                    DvnValue::Sankhya(42),
                    DvnValue::Vakya("Sankhya".to_string()),
                ],
            )
            .unwrap();
        assert_eq!(result, DvnValue::Satya(true));
    }

    #[test]
    fn test_janayati_type() {
        let reg = DhatuRegistry::new();
        let result = reg
            .call("janayati", vec![DvnValue::Vakya("hello".to_string())])
            .unwrap();
        assert_eq!(result, DvnValue::Vakya("Vakya".to_string()));
    }

    #[test]
    fn test_tulayati_abs() {
        let reg = DhatuRegistry::new();
        let result = reg.call("tulayati", vec![DvnValue::Sankhya(-99)]).unwrap();
        assert_eq!(result, DvnValue::Sankhya(99));
    }

    #[test]
    fn test_dvibhajati_bin() {
        let reg = DhatuRegistry::new();
        let result = reg.call("dvibhajati", vec![DvnValue::Sankhya(10)]).unwrap();
        assert_eq!(result, DvnValue::Vakya("0b1010".to_string()));
    }

    #[test]
    fn test_sarvayati_all() {
        let reg = DhatuRegistry::new();
        let result = reg
            .call(
                "sarvayati",
                vec![DvnValue::Suchi(vec![
                    DvnValue::Satya(true),
                    DvnValue::Satya(true),
                    DvnValue::Satya(true),
                ])],
            )
            .unwrap();
        assert_eq!(result, DvnValue::Satya(true));
    }

    #[test]
    fn test_samyojayati_join() {
        let reg = DhatuRegistry::new();
        let result = reg
            .call(
                "samyojayati",
                vec![
                    DvnValue::Vakya(", ".to_string()),
                    DvnValue::Suchi(vec![
                        DvnValue::Vakya("deva".to_string()),
                        DvnValue::Vakya("vani".to_string()),
                        DvnValue::Vakya("bhasha".to_string()),
                    ]),
                ],
            )
            .unwrap();
        assert_eq!(result, DvnValue::Vakya("deva, vani, bhasha".to_string()));
    }

    #[test]
    fn test_vibhajati_split() {
        let reg = DhatuRegistry::new();
        let result = reg
            .call(
                "vibhajati",
                vec![
                    DvnValue::Vakya("a:b:c".to_string()),
                    DvnValue::Vakya(":".to_string()),
                ],
            )
            .unwrap();
        assert_eq!(
            result,
            DvnValue::Suchi(vec![
                DvnValue::Vakya("a".to_string()),
                DvnValue::Vakya("b".to_string()),
                DvnValue::Vakya("c".to_string()),
            ])
        );
    }

    #[test]
    fn test_anukramati_slice() {
        let reg = DhatuRegistry::new();
        let result = reg
            .call(
                "anukramati",
                vec![
                    DvnValue::Suchi(vec![
                        DvnValue::Sankhya(0),
                        DvnValue::Sankhya(1),
                        DvnValue::Sankhya(2),
                        DvnValue::Sankhya(3),
                        DvnValue::Sankhya(4),
                    ]),
                    DvnValue::Sankhya(1),
                    DvnValue::Sankhya(4),
                ],
            )
            .unwrap();
        assert_eq!(
            result,
            DvnValue::Suchi(vec![
                DvnValue::Sankhya(1),
                DvnValue::Sankhya(2),
                DvnValue::Sankhya(3),
            ])
        );
    }

    #[test]
    fn test_prelude_init() {
        use crate::prelude::devvani_prelude;
        let prelude = devvani_prelude();
        assert!(prelude.is_stdlib_dhatu("vadati"));
        assert!(prelude.is_stdlib_dhatu("ganavati"));
        assert!(prelude.is_stdlib_dhatu("tulayati"));
        assert!(!prelude.is_stdlib_dhatu("unknown_fn"));
    }
}
