use crate::{DvnValue, DhatuFn, StdlibError};
use crate::registry::DhatuRegistry;

pub struct Kramate;
impl DhatuFn for Kramate {
    fn name(&self) -> &'static str { "kramate" }
    fn sanskrit_root(&self) -> &'static str { "kram" }
    fn python_equivalent(&self) -> &'static str { "range" }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        let (start, stop, step) = match args.len() {
            1 => (0, match args[0] { DvnValue::Sankhya(n) => n, _ => return Err(StdlibError::TypeError { dhatu: "kramate".into(), expected: "Sankhya".into(), got: format!("{:?}", args[0]) }) }, 1),
            2 => (
                match args[0] { DvnValue::Sankhya(n) => n, _ => return Err(StdlibError::TypeError { dhatu: "kramate".into(), expected: "Sankhya".into(), got: format!("{:?}", args[0]) }) },
                match args[1] { DvnValue::Sankhya(n) => n, _ => return Err(StdlibError::TypeError { dhatu: "kramate".into(), expected: "Sankhya".into(), got: format!("{:?}", args[1]) }) },
                1
            ),
            3 => (
                match args[0] { DvnValue::Sankhya(n) => n, _ => return Err(StdlibError::TypeError { dhatu: "kramate".into(), expected: "Sankhya".into(), got: format!("{:?}", args[0]) }) },
                match args[1] { DvnValue::Sankhya(n) => n, _ => return Err(StdlibError::TypeError { dhatu: "kramate".into(), expected: "Sankhya".into(), got: format!("{:?}", args[1]) }) },
                match args[2] { DvnValue::Sankhya(n) => n, _ => return Err(StdlibError::TypeError { dhatu: "kramate".into(), expected: "Sankhya".into(), got: format!("{:?}", args[2]) }) }
            ),
            _ => return Err(StdlibError::ArgCount { dhatu: "kramate".into(), expected: 1, got: args.len() }),
        };

        let mut values = Vec::new();
        let mut curr = start;
        if step > 0 {
            while curr < stop {
                values.push(DvnValue::Sankhya(curr));
                curr += step;
            }
        } else if step < 0 {
            while curr > stop {
                values.push(DvnValue::Sankhya(curr));
                curr += step;
            }
        }
        Ok(DvnValue::Suchi(values))
    }
}

pub struct Yugmayati;
impl DhatuFn for Yugmayati {
    fn name(&self) -> &'static str { "yugmayati" }
    fn sanskrit_root(&self) -> &'static str { "yuj" }
    fn python_equivalent(&self) -> &'static str { "zip" }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.len() < 2 {
            return Err(StdlibError::ArgCount { dhatu: "yugmayati".into(), expected: 2, got: args.len() });
        }
        let list1 = match &args[0] { DvnValue::Suchi(v) => v, _ => return Err(StdlibError::TypeError { dhatu: "yugmayati".into(), expected: "Suchi".into(), got: format!("{:?}", args[0]) }) };
        let list2 = match &args[1] { DvnValue::Suchi(v) => v, _ => return Err(StdlibError::TypeError { dhatu: "yugmayati".into(), expected: "Suchi".into(), got: format!("{:?}", args[1]) }) };
        
        let mut result = Vec::new();
        for i in 0..list1.len().min(list2.len()) {
            result.push(DvnValue::Sutram(vec![list1[i].clone(), list2[i].clone()]));
        }
        Ok(DvnValue::Suchi(result))
    }
}

pub struct Citrayati;
impl DhatuFn for Citrayati {
    fn name(&self) -> &'static str { "citrayati" }
    fn sanskrit_root(&self) -> &'static str { "citr" }
    fn python_equivalent(&self) -> &'static str { "map" }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.len() < 2 {
            return Err(StdlibError::ArgCount { dhatu: "citrayati".into(), expected: 2, got: args.len() });
        }
        let fn_name = match &args[0] { DvnValue::Kriya(s) => s, _ => return Err(StdlibError::TypeError { dhatu: "citrayati".into(), expected: "Kriya".into(), got: format!("{:?}", args[0]) }) };
        let list = match &args[1] { DvnValue::Suchi(v) => v, _ => return Err(StdlibError::TypeError { dhatu: "citrayati".into(), expected: "Suchi".into(), got: format!("{:?}", args[1]) }) };
        
        let registry = DhatuRegistry::new();
        let mut result = Vec::new();
        for item in list {
            result.push(registry.call(fn_name, vec![item.clone()])?);
        }
        Ok(DvnValue::Suchi(result))
    }
}

pub struct Chinati;
impl DhatuFn for Chinati {
    fn name(&self) -> &'static str { "chinati" }
    fn sanskrit_root(&self) -> &'static str { "ci" }
    fn python_equivalent(&self) -> &'static str { "filter" }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.len() < 2 {
            return Err(StdlibError::ArgCount { dhatu: "chinati".into(), expected: 2, got: args.len() });
        }
        let fn_name = match &args[0] { DvnValue::Kriya(s) => s, _ => return Err(StdlibError::TypeError { dhatu: "chinati".into(), expected: "Kriya".into(), got: format!("{:?}", args[0]) }) };
        let list = match &args[1] { DvnValue::Suchi(v) => v, _ => return Err(StdlibError::TypeError { dhatu: "chinati".into(), expected: "Suchi".into(), got: format!("{:?}", args[1]) }) };
        
        let registry = DhatuRegistry::new();
        let mut result = Vec::new();
        for item in list {
            let res = registry.call(fn_name, vec![item.clone()])?;
            let is_true = match res {
                DvnValue::Satya(b) => b,
                DvnValue::Sankhya(n) => n != 0,
                _ => true,
            };
            if is_true {
                result.push(item.clone());
            }
        }
        Ok(DvnValue::Suchi(result))
    }
}

pub struct Kramankati;
impl DhatuFn for Kramankati {
    fn name(&self) -> &'static str { "kramankati" }
    fn sanskrit_root(&self) -> &'static str { "kram" }
    fn python_equivalent(&self) -> &'static str { "enumerate" }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() { return Err(StdlibError::ArgCount { dhatu: "kramankati".into(), expected: 1, got: 0 }); }
        let list = match &args[0] { DvnValue::Suchi(v) => v, _ => return Err(StdlibError::TypeError { dhatu: "kramankati".into(), expected: "Suchi".into(), got: format!("{:?}", args[0]) }) };
        
        let mut result = Vec::new();
        for (i, val) in list.iter().enumerate() {
            result.push(DvnValue::Sutram(vec![DvnValue::Sankhya(i as i64), val.clone()]));
        }
        Ok(DvnValue::Suchi(result))
    }
}

pub struct Uttamayati;
impl DhatuFn for Uttamayati {
    fn name(&self) -> &'static str { "uttamayati" }
    fn sanskrit_root(&self) -> &'static str { "ud" }
    fn python_equivalent(&self) -> &'static str { "max" }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() { return Err(StdlibError::ArgCount { dhatu: "uttamayati".into(), expected: 1, got: 0 }); }
        let list = match &args[0] { DvnValue::Suchi(v) => v, _ => return Err(StdlibError::TypeError { dhatu: "uttamayati".into(), expected: "Suchi".into(), got: format!("{:?}", args[0]) }) };
        if list.is_empty() { return Ok(DvnValue::Shunya); }
        
        let mut max_val = list[0].clone();
        for item in list.iter().skip(1) {
            match (&max_val, item) {
                (DvnValue::Sankhya(a), DvnValue::Sankhya(b)) => if b > a { max_val = item.clone(); },
                (DvnValue::Dasha(a), DvnValue::Dasha(b)) => if b > a { max_val = item.clone(); },
                _ => {}
            }
        }
        Ok(max_val)
    }
}

pub struct Avamayati;
impl DhatuFn for Avamayati {
    fn name(&self) -> &'static str { "avamayati" }
    fn sanskrit_root(&self) -> &'static str { "ava" }
    fn python_equivalent(&self) -> &'static str { "min" }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() { return Err(StdlibError::ArgCount { dhatu: "avamayati".into(), expected: 1, got: 0 }); }
        let list = match &args[0] { DvnValue::Suchi(v) => v, _ => return Err(StdlibError::TypeError { dhatu: "avamayati".into(), expected: "Suchi".into(), got: format!("{:?}", args[0]) }) };
        if list.is_empty() { return Ok(DvnValue::Shunya); }
        
        let mut min_val = list[0].clone();
        for item in list.iter().skip(1) {
            match (&min_val, item) {
                (DvnValue::Sankhya(a), DvnValue::Sankhya(b)) => if b < a { min_val = item.clone(); },
                (DvnValue::Dasha(a), DvnValue::Dasha(b)) => if b < a { min_val = item.clone(); },
                _ => {}
            }
        }
        Ok(min_val)
    }
}

pub struct Yojayati;
impl DhatuFn for Yojayati {
    fn name(&self) -> &'static str { "yojayati" }
    fn sanskrit_root(&self) -> &'static str { "yuj" }
    fn python_equivalent(&self) -> &'static str { "sum" }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() { return Err(StdlibError::ArgCount { dhatu: "yojayati".into(), expected: 1, got: 0 }); }
        let list = match &args[0] { DvnValue::Suchi(v) => v, _ => return Err(StdlibError::TypeError { dhatu: "yojayati".into(), expected: "Suchi".into(), got: format!("{:?}", args[0]) }) };
        
        let mut sum_i = 0;
        let mut sum_f = 0.0;
        let mut is_float = false;
        
        for item in list {
            match item {
                DvnValue::Sankhya(n) => {
                    if is_float { sum_f += *n as f64; } else { sum_i += n; }
                }
                DvnValue::Dasha(f) => {
                    if !is_float {
                        is_float = true;
                        sum_f = sum_i as f64;
                    }
                    sum_f += f;
                }
                _ => return Err(StdlibError::TypeError { dhatu: "yojayati".into(), expected: "numeric".into(), got: format!("{:?}", item) }),
            }
        }
        
        if is_float { Ok(DvnValue::Dasha(sum_f)) } else { Ok(DvnValue::Sankhya(sum_i)) }
    }
}

pub struct Kramayati;
impl DhatuFn for Kramayati {
    fn name(&self) -> &'static str { "kramayati" }
    fn sanskrit_root(&self) -> &'static str { "kram" }
    fn python_equivalent(&self) -> &'static str { "sorted" }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() { return Err(StdlibError::ArgCount { dhatu: "kramayati".into(), expected: 1, got: 0 }); }
        let mut list = match &args[0] { DvnValue::Suchi(v) => v.clone(), _ => return Err(StdlibError::TypeError { dhatu: "kramayati".into(), expected: "Suchi".into(), got: format!("{:?}", args[0]) }) };
        
        list.sort_by(|a, b| match (a, b) {
            (DvnValue::Sankhya(x), DvnValue::Sankhya(y)) => x.cmp(y),
            (DvnValue::Vakya(x), DvnValue::Vakya(y)) => x.cmp(y),
            _ => std::cmp::Ordering::Equal,
        });
        Ok(DvnValue::Suchi(list))
    }
}

pub struct Viparitayati;
impl DhatuFn for Viparitayati {
    fn name(&self) -> &'static str { "viparitayati" }
    fn sanskrit_root(&self) -> &'static str { "vi-pari-i" }
    fn python_equivalent(&self) -> &'static str { "reversed" }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() { return Err(StdlibError::ArgCount { dhatu: "viparitayati".into(), expected: 1, got: 0 }); }
        let mut list = match &args[0] { DvnValue::Suchi(v) => v.clone(), _ => return Err(StdlibError::TypeError { dhatu: "viparitayati".into(), expected: "Suchi".into(), got: format!("{:?}", args[0]) }) };
        list.reverse();
        Ok(DvnValue::Suchi(list))
    }
}

pub struct Ganavati;
impl DhatuFn for Ganavati {
    fn name(&self) -> &'static str { "ganavati" }
    fn sanskrit_root(&self) -> &'static str { "gan" }
    fn python_equivalent(&self) -> &'static str { "len" }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() { return Err(StdlibError::ArgCount { dhatu: "ganavati".into(), expected: 1, got: 0 }); }
        match &args[0] {
            DvnValue::Suchi(v) => Ok(DvnValue::Sankhya(v.len() as i64)),
            DvnValue::Vakya(s) => Ok(DvnValue::Sankhya(s.chars().count() as i64)),
            DvnValue::Sutram(v) => Ok(DvnValue::Sankhya(v.len() as i64)),
            DvnValue::Kosha(m) => Ok(DvnValue::Sankhya(m.len() as i64)),
            DvnValue::Samuha(s) => Ok(DvnValue::Sankhya(s.len() as i64)),
            _ => Err(StdlibError::TypeError { dhatu: "ganavati".into(), expected: "collection".into(), got: format!("{:?}", args[0]) }),
        }
    }
}
