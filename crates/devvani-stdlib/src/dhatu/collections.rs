use crate::{DvnValue, DhatuFn, StdlibError};
use std::collections::{HashMap, HashSet};

pub struct Sucayati;
impl DhatuFn for Sucayati {
    fn name(&self) -> &'static str { "sucayati" }
    fn sanskrit_root(&self) -> &'static str { "suc" }
    fn python_equivalent(&self) -> &'static str { "list" }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() {
            return Ok(DvnValue::Suchi(Vec::new()));
        }
        match &args[0] {
            DvnValue::Suchi(v) => Ok(DvnValue::Suchi(v.clone())),
            DvnValue::Sutram(v) => Ok(DvnValue::Suchi(v.clone())),
            _ => Ok(DvnValue::Suchi(vec![args[0].clone()])),
        }
    }
}

pub struct Sutrayati;
impl DhatuFn for Sutrayati {
    fn name(&self) -> &'static str { "sutrayati" }
    fn sanskrit_root(&self) -> &'static str { "sutr" }
    fn python_equivalent(&self) -> &'static str { "tuple" }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        Ok(DvnValue::Sutram(args))
    }
}

pub struct Koshayati;
impl DhatuFn for Koshayati {
    fn name(&self) -> &'static str { "koshayati" }
    fn sanskrit_root(&self) -> &'static str { "kush" }
    fn python_equivalent(&self) -> &'static str { "dict" }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        let mut map = HashMap::new();
        for arg in args {
            if let DvnValue::Sutram(pair) = arg {
                if pair.len() == 2 {
                    if let DvnValue::Vakya(key) = &pair[0] {
                        map.insert(key.clone(), pair[1].clone());
                    } else {
                        return Err(StdlibError::TypeError { dhatu: "koshayati".into(), expected: "Vakya key".into(), got: format!("{:?}", pair[0]) });
                    }
                } else {
                    return Err(StdlibError::TypeError { dhatu: "koshayati".into(), expected: "pair".into(), got: format!("length {}", pair.len()) });
                }
            } else {
                return Err(StdlibError::TypeError { dhatu: "koshayati".into(), expected: "Sutram pair".into(), got: format!("{:?}", arg) });
            }
        }
        Ok(DvnValue::Kosha(map))
    }
}

pub struct Sthapayati;
impl DhatuFn for Sthapayati {
    fn name(&self) -> &'static str { "sthapayati" }
    fn sanskrit_root(&self) -> &'static str { "stha" }
    fn python_equivalent(&self) -> &'static str { "set" }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() {
            return Ok(DvnValue::Samuha(HashSet::new()));
        }
        let mut set = HashSet::new();
        let items = match &args[0] {
            DvnValue::Suchi(v) => v,
            _ => return Err(StdlibError::TypeError { dhatu: "sthapayati".into(), expected: "Suchi".into(), got: format!("{:?}", args[0]) }),
        };
        for item in items {
            set.insert(match item {
                DvnValue::Vakya(s) => s.clone(),
                DvnValue::Sankhya(n) => n.to_string(),
                _ => format!("{:?}", item),
            });
        }
        Ok(DvnValue::Samuha(set))
    }
}
