use crate::{DvnValue, DhatuFn, StdlibError};

pub struct Tulayati;
impl DhatuFn for Tulayati {
    fn name(&self) -> &'static str { "tulayati" }
    fn sanskrit_root(&self) -> &'static str { "tul" }
    fn python_equivalent(&self) -> &'static str { "abs" }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() { return Err(StdlibError::ArgCount { dhatu: "tulayati".into(), expected: 1, got: 0 }); }
        match &args[0] {
            DvnValue::Sankhya(n) => Ok(DvnValue::Sankhya(n.abs())),
            DvnValue::Dasha(f) => Ok(DvnValue::Dasha(f.abs())),
            _ => Err(StdlibError::TypeError { dhatu: "tulayati".into(), expected: "numeric".into(), got: format!("{:?}", args[0]) }),
        }
    }
}

pub struct Seshayati;
impl DhatuFn for Seshayati {
    fn name(&self) -> &'static str { "seshayati" }
    fn sanskrit_root(&self) -> &'static str { "sish" }
    fn python_equivalent(&self) -> &'static str { "divmod" }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.len() < 2 { return Err(StdlibError::ArgCount { dhatu: "seshayati".into(), expected: 2, got: args.len() }); }
        let a = match args[0] { DvnValue::Sankhya(n) => n, _ => return Err(StdlibError::TypeError { dhatu: "seshayati".into(), expected: "Sankhya".into(), got: format!("{:?}", args[0]) }) };
        let b = match args[1] { DvnValue::Sankhya(n) => n, _ => return Err(StdlibError::TypeError { dhatu: "seshayati".into(), expected: "Sankhya".into(), got: format!("{:?}", args[1]) }) };
        if b == 0 { return Err(StdlibError::IoError { dhatu: "seshayati".into(), msg: "division by zero".into() }); }
        Ok(DvnValue::Sutram(vec![DvnValue::Sankhya(a / b), DvnValue::Sankhya(a % b)]))
    }
}

pub struct Vardhayati;
impl DhatuFn for Vardhayati {
    fn name(&self) -> &'static str { "vardhayati" }
    fn sanskrit_root(&self) -> &'static str { "vrdh" }
    fn python_equivalent(&self) -> &'static str { "pow" }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.len() < 2 { return Err(StdlibError::ArgCount { dhatu: "vardhayati".into(), expected: 2, got: args.len() }); }
        let base = match args[0] { DvnValue::Sankhya(n) => n, _ => return Err(StdlibError::TypeError { dhatu: "vardhayati".into(), expected: "Sankhya".into(), got: format!("{:?}", args[0]) }) };
        let exp = match args[1] { DvnValue::Sankhya(n) => n, _ => return Err(StdlibError::TypeError { dhatu: "vardhayati".into(), expected: "Sankhya".into(), got: format!("{:?}", args[1]) }) };
        let modulus = args.get(2).and_then(|v| if let DvnValue::Sankhya(n) = v { Some(*n) } else { None });

        let res = if let Some(m) = modulus {
            if m == 0 { return Err(StdlibError::IoError { dhatu: "vardhayati".into(), msg: "modulo by zero".into() }); }
            let mut base = base % m;
            let mut exp = exp;
            let mut result = 1;
            while exp > 0 {
                if exp % 2 == 1 { result = (result * base) % m; }
                base = (base * base) % m;
                exp /= 2;
            }
            result
        } else {
            base.pow(exp as u32)
        };
        Ok(DvnValue::Sankhya(res))
    }
}

pub struct Purnayati;
impl DhatuFn for Purnayati {
    fn name(&self) -> &'static str { "purnayati" }
    fn sanskrit_root(&self) -> &'static str { "pr" }
    fn python_equivalent(&self) -> &'static str { "round" }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() { return Err(StdlibError::ArgCount { dhatu: "purnayati".into(), expected: 1, got: 0 }); }
        let f = match args[0] { DvnValue::Dasha(n) => n, _ => return Err(StdlibError::TypeError { dhatu: "purnayati".into(), expected: "Dasha".into(), got: format!("{:?}", args[0]) }) };
        let places = args.get(1).and_then(|v| if let DvnValue::Sankhya(n) = v { Some(*n) } else { None }).unwrap_or(0);
        
        let multiplier = 10f64.powi(places as i32);
        Ok(DvnValue::Dasha((f * multiplier).round() / multiplier))
    }
}

pub struct Dvibhajati;
impl DhatuFn for Dvibhajati {
    fn name(&self) -> &'static str { "dvibhajati" }
    fn sanskrit_root(&self) -> &'static str { "bhaj" }
    fn python_equivalent(&self) -> &'static str { "bin" }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() { return Err(StdlibError::ArgCount { dhatu: "dvibhajati".into(), expected: 1, got: 0 }); }
        let n = match args[0] { DvnValue::Sankhya(n) => n, _ => return Err(StdlibError::TypeError { dhatu: "dvibhajati".into(), expected: "Sankhya".into(), got: format!("{:?}", args[0]) }) };
        Ok(DvnValue::Vakya(format!("0b{:b}", n)))
    }
}

pub struct Astabhajati;
impl DhatuFn for Astabhajati {
    fn name(&self) -> &'static str { "astabhajati" }
    fn sanskrit_root(&self) -> &'static str { "bhaj" }
    fn python_equivalent(&self) -> &'static str { "oct" }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() { return Err(StdlibError::ArgCount { dhatu: "astabhajati".into(), expected: 1, got: 0 }); }
        let n = match args[0] { DvnValue::Sankhya(n) => n, _ => return Err(StdlibError::TypeError { dhatu: "astabhajati".into(), expected: "Sankhya".into(), got: format!("{:?}", args[0]) }) };
        Ok(DvnValue::Vakya(format!("0o{:o}", n)))
    }
}

pub struct Sodasabhajati;
impl DhatuFn for Sodasabhajati {
    fn name(&self) -> &'static str { "sodasabhajati" }
    fn sanskrit_root(&self) -> &'static str { "bhaj" }
    fn python_equivalent(&self) -> &'static str { "hex" }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() { return Err(StdlibError::ArgCount { dhatu: "sodasabhajati".into(), expected: 1, got: 0 }); }
        let n = match args[0] { DvnValue::Sankhya(n) => n, _ => return Err(StdlibError::TypeError { dhatu: "sodasabhajati".into(), expected: "Sankhya".into(), got: format!("{:?}", args[0]) }) };
        Ok(DvnValue::Vakya(format!("0x{:x}", n)))
    }
}

pub struct Varnayati;
impl DhatuFn for Varnayati {
    fn name(&self) -> &'static str { "varnayati" }
    fn sanskrit_root(&self) -> &'static str { "varn" }
    fn python_equivalent(&self) -> &'static str { "chr" }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() { return Err(StdlibError::ArgCount { dhatu: "varnayati".into(), expected: 1, got: 0 }); }
        let n = match args[0] { DvnValue::Sankhya(n) => n, _ => return Err(StdlibError::TypeError { dhatu: "varnayati".into(), expected: "Sankhya".into(), got: format!("{:?}", args[0]) }) };
        std::char::from_u32(n as u32)
            .map(|c| DvnValue::Vakya(c.to_string()))
            .ok_or_else(|| StdlibError::ConversionError { dhatu: "varnayati".into(), msg: "invalid codepoint".into() })
    }
}

pub struct Samkhyati;
impl DhatuFn for Samkhyati {
    fn name(&self) -> &'static str { "samkhyati" }
    fn sanskrit_root(&self) -> &'static str { "khya" }
    fn python_equivalent(&self) -> &'static str { "count" }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.len() < 2 { return Err(StdlibError::ArgCount { dhatu: "samkhyati".into(), expected: 2, got: args.len() }); }
        let list = match &args[0] { DvnValue::Suchi(v) => v, _ => return Err(StdlibError::TypeError { dhatu: "samkhyati".into(), expected: "Suchi".into(), got: format!("{:?}", args[0]) }) };
        let target = &args[1];
        let count = list.iter().filter(|&x| x == target).count();
        Ok(DvnValue::Sankhya(count as i64))
    }
}
