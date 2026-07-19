use crate::{DhatuFn, DvnValue, StdlibError};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct Asti;
impl DhatuFn for Asti {
    fn name(&self) -> &'static str {
        "asti"
    }
    fn sanskrit_root(&self) -> &'static str {
        "as"
    }
    fn python_equivalent(&self) -> &'static str {
        "isinstance"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.len() < 2 {
            return Err(StdlibError::ArgCount {
                dhatu: "asti".into(),
                expected: 2,
                got: args.len(),
            });
        }
        let type_name = match &args[1] {
            DvnValue::Vakya(s) => s.as_str(),
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "asti".into(),
                    expected: "Vakya".into(),
                    got: format!("{:?}", args[1]),
                })
            }
        };
        let actual_type = match &args[0] {
            DvnValue::Sankhya(_) => "Sankhya",
            DvnValue::Dasha(_) => "Dasha",
            DvnValue::Vakya(_) => "Vakya",
            DvnValue::Satya(_) => "Satya",
            DvnValue::Suchi(_) => "Suchi",
            DvnValue::Sutram(_) => "Sutram",
            DvnValue::Kosha(_) => "Kosha",
            DvnValue::Samuha(_) => "Samuha",
            DvnValue::Kriya(_) => "Kriya",
            DvnValue::Shunya => "Shunya",
        };
        Ok(DvnValue::Satya(actual_type == type_name))
    }
}

pub struct Vidyate;
impl DhatuFn for Vidyate {
    fn name(&self) -> &'static str {
        "vidyate"
    }
    fn sanskrit_root(&self) -> &'static str {
        "vid"
    }
    fn python_equivalent(&self) -> &'static str {
        "hasattr"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.len() < 2 {
            return Err(StdlibError::ArgCount {
                dhatu: "vidyate".into(),
                expected: 2,
                got: args.len(),
            });
        }
        let kosha = match &args[0] {
            DvnValue::Kosha(m) => m,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "vidyate".into(),
                    expected: "Kosha".into(),
                    got: format!("{:?}", args[0]),
                })
            }
        };
        let key = match &args[1] {
            DvnValue::Vakya(s) => s,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "vidyate".into(),
                    expected: "Vakya".into(),
                    got: format!("{:?}", args[1]),
                })
            }
        };
        Ok(DvnValue::Satya(kosha.contains_key(key)))
    }
}

pub struct Grhnati;
impl DhatuFn for Grhnati {
    fn name(&self) -> &'static str {
        "grhnati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "grah"
    }
    fn python_equivalent(&self) -> &'static str {
        "getattr"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.len() < 2 {
            return Err(StdlibError::ArgCount {
                dhatu: "grhnati".into(),
                expected: 2,
                got: args.len(),
            });
        }
        let kosha = match &args[0] {
            DvnValue::Kosha(m) => m,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "grhnati".into(),
                    expected: "Kosha".into(),
                    got: format!("{:?}", args[0]),
                })
            }
        };
        let key = match &args[1] {
            DvnValue::Vakya(s) => s,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "grhnati".into(),
                    expected: "Vakya".into(),
                    got: format!("{:?}", args[1]),
                })
            }
        };
        let default = args.get(2).cloned().unwrap_or(DvnValue::Shunya);
        Ok(kosha.get(key).cloned().unwrap_or(default))
    }
}

pub struct SthapayatiAttr;
impl DhatuFn for SthapayatiAttr {
    fn name(&self) -> &'static str {
        "sthapayati_attr"
    }
    fn sanskrit_root(&self) -> &'static str {
        "stha"
    }
    fn python_equivalent(&self) -> &'static str {
        "setattr"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.len() < 3 {
            return Err(StdlibError::ArgCount {
                dhatu: "sthapayati_attr".into(),
                expected: 3,
                got: args.len(),
            });
        }
        let mut kosha = match &args[0] {
            DvnValue::Kosha(m) => m.clone(),
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "sthapayati_attr".into(),
                    expected: "Kosha".into(),
                    got: format!("{:?}", args[0]),
                })
            }
        };
        let key = match &args[1] {
            DvnValue::Vakya(s) => s.clone(),
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "sthapayati_attr".into(),
                    expected: "Vakya".into(),
                    got: format!("{:?}", args[1]),
                })
            }
        };
        kosha.insert(key, args[2].clone());
        Ok(DvnValue::Kosha(kosha))
    }
}

pub struct Janayati;
impl DhatuFn for Janayati {
    fn name(&self) -> &'static str {
        "janayati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "jan"
    }
    fn python_equivalent(&self) -> &'static str {
        "type"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() {
            return Err(StdlibError::ArgCount {
                dhatu: "janayati".into(),
                expected: 1,
                got: 0,
            });
        }
        let type_name = match &args[0] {
            DvnValue::Sankhya(_) => "Sankhya",
            DvnValue::Dasha(_) => "Dasha",
            DvnValue::Vakya(_) => "Vakya",
            DvnValue::Satya(_) => "Satya",
            DvnValue::Suchi(_) => "Suchi",
            DvnValue::Sutram(_) => "Sutram",
            DvnValue::Kosha(_) => "Kosha",
            DvnValue::Samuha(_) => "Samuha",
            DvnValue::Kriya(_) => "Kriya",
            DvnValue::Shunya => "Shunya",
        };
        Ok(DvnValue::Vakya(type_name.into()))
    }
}

pub struct Avatarayati;
impl DhatuFn for Avatarayati {
    fn name(&self) -> &'static str {
        "avatarayati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "tr"
    }
    fn python_equivalent(&self) -> &'static str {
        "id"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() {
            return Err(StdlibError::ArgCount {
                dhatu: "avatarayati".into(),
                expected: 1,
                got: 0,
            });
        }
        let mut hasher = DefaultHasher::new();
        format!("{:?}", args[0]).hash(&mut hasher);
        Ok(DvnValue::Sankhya(hasher.finish() as i64))
    }
}

pub struct Mapayati;
impl DhatuFn for Mapayati {
    fn name(&self) -> &'static str {
        "mapayati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "ma"
    }
    fn python_equivalent(&self) -> &'static str {
        "hash"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() {
            return Err(StdlibError::ArgCount {
                dhatu: "mapayati".into(),
                expected: 1,
                got: 0,
            });
        }
        let mut hasher = DefaultHasher::new();
        format!("{:?}", args[0]).hash(&mut hasher);
        Ok(DvnValue::Sankhya(hasher.finish() as i64))
    }
}

pub struct Kalpayati;
impl DhatuFn for Kalpayati {
    fn name(&self) -> &'static str {
        "kalpayati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "klp"
    }
    fn python_equivalent(&self) -> &'static str {
        "callable"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() {
            return Err(StdlibError::ArgCount {
                dhatu: "kalpayati".into(),
                expected: 1,
                got: 0,
            });
        }
        Ok(DvnValue::Satya(matches!(&args[0], DvnValue::Kriya(_))))
    }
}
