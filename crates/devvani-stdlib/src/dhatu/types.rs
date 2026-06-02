use crate::{DvnValue, DhatuFn, StdlibError};

pub struct PariNameti;
impl DhatuFn for PariNameti {
    fn name(&self) -> &'static str { "parinameti" }
    fn sanskrit_root(&self) -> &'static str { "nam" }
    fn python_equivalent(&self) -> &'static str { "int" }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() {
            return Err(StdlibError::ArgCount { dhatu: "parinameti".into(), expected: 1, got: 0 });
        }
        match &args[0] {
            DvnValue::Sankhya(n) => Ok(DvnValue::Sankhya(*n)),
            DvnValue::Dasha(f) => Ok(DvnValue::Sankhya(*f as i64)),
            DvnValue::Vakya(s) => s.parse::<i64>().map(DvnValue::Sankhya).map_err(|e| StdlibError::ConversionError { 
                dhatu: "parinameti".into(), msg: e.to_string() 
            }),
            DvnValue::Satya(b) => Ok(DvnValue::Sankhya(if *b { 1 } else { 0 })),
            _ => Err(StdlibError::TypeError { dhatu: "parinameti".into(), expected: "convertible to Sankhya".into(), got: format!("{:?}", args[0]) }),
        }
    }
}

pub struct Vakyayati;
impl DhatuFn for Vakyayati {
    fn name(&self) -> &'static str { "vakyayati" }
    fn sanskrit_root(&self) -> &'static str { "vac" }
    fn python_equivalent(&self) -> &'static str { "str" }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() {
            return Err(StdlibError::ArgCount { dhatu: "vakyayati".into(), expected: 1, got: 0 });
        }
        Ok(DvnValue::Vakya(match &args[0] {
            DvnValue::Vakya(s) => s.clone(),
            DvnValue::Sankhya(n) => n.to_string(),
            DvnValue::Dasha(f) => f.to_string(),
            DvnValue::Satya(b) => b.to_string(),
            _ => format!("{:?}", args[0]),
        }))
    }
}

pub struct Kampayati;
impl DhatuFn for Kampayati {
    fn name(&self) -> &'static str { "kampayati" }
    fn sanskrit_root(&self) -> &'static str { "kamp" }
    fn python_equivalent(&self) -> &'static str { "float" }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() {
            return Err(StdlibError::ArgCount { dhatu: "kampayati".into(), expected: 1, got: 0 });
        }
        match &args[0] {
            DvnValue::Dasha(f) => Ok(DvnValue::Dasha(*f)),
            DvnValue::Sankhya(n) => Ok(DvnValue::Dasha(*n as f64)),
            DvnValue::Vakya(s) => s.parse::<f64>().map(DvnValue::Dasha).map_err(|e| StdlibError::ConversionError { 
                dhatu: "kampayati".into(), msg: e.to_string() 
            }),
            _ => Err(StdlibError::TypeError { dhatu: "kampayati".into(), expected: "convertible to Dasha".into(), got: format!("{:?}", args[0]) }),
        }
    }
}

pub struct Dvayati;
impl DhatuFn for Dvayati {
    fn name(&self) -> &'static str { "dvayati" }
    fn sanskrit_root(&self) -> &'static str { "dvi" }
    fn python_equivalent(&self) -> &'static str { "bool" }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() {
            return Err(StdlibError::ArgCount { dhatu: "dvayati".into(), expected: 1, got: 0 });
        }
        Ok(DvnValue::Satya(match &args[0] {
            DvnValue::Satya(b) => *b,
            DvnValue::Sankhya(n) => *n != 0,
            DvnValue::Dasha(f) => *f != 0.0,
            DvnValue::Vakya(s) => !s.is_empty(),
            DvnValue::Suchi(v) => !v.is_empty(),
            DvnValue::Shunya => false,
            _ => true,
        }))
    }
}

pub struct PrakaShati;
impl DhatuFn for PrakaShati {
    fn name(&self) -> &'static str { "prakashati" }
    fn sanskrit_root(&self) -> &'static str { "kash" }
    fn python_equivalent(&self) -> &'static str { "repr" }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() {
            return Err(StdlibError::ArgCount { dhatu: "prakashati".into(), expected: 1, got: 0 });
        }
        Ok(DvnValue::Vakya(format!("{:?}", args[0])))
    }
}

pub struct Manayati;
impl DhatuFn for Manayati {
    fn name(&self) -> &'static str { "manayati" }
    fn sanskrit_root(&self) -> &'static str { "man" }
    fn python_equivalent(&self) -> &'static str { "ord" }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() {
            return Err(StdlibError::ArgCount { dhatu: "manayati".into(), expected: 1, got: 0 });
        }
        match &args[0] {
            DvnValue::Vakya(s) => {
                if s.chars().count() == 1 {
                    Ok(DvnValue::Sankhya(s.chars().next().unwrap() as i64))
                } else {
                    Err(StdlibError::ConversionError { dhatu: "manayati".into(), msg: "Expected a single character string".into() })
                }
            },
            _ => Err(StdlibError::TypeError { dhatu: "manayati".into(), expected: "Vakya (char)".into(), got: format!("{:?}", args[0]) }),
        }
    }
}
