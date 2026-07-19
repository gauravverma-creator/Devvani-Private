use crate::{DhatuFn, DvnValue, StdlibError};
use std::collections::{HashMap, HashSet};

// --- Execution ---

pub struct Arabhati;
impl DhatuFn for Arabhati {
    fn name(&self) -> &'static str {
        "arabhati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "rabh"
    }
    fn python_equivalent(&self) -> &'static str {
        "exec"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() {
            return Err(StdlibError::ArgCount {
                dhatu: "arabhati".into(),
                expected: 1,
                got: 0,
            });
        }
        let code = match &args[0] {
            DvnValue::Vakya(s) => s,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "arabhati".into(),
                    expected: "Vakya".into(),
                    got: format!("{:?}", args[0]),
                })
            }
        };

        // Safety: only allow vadati(...) calls
        if code.contains("vadati(") {
            let parts: Vec<&str> = code.split("vadati(").collect();
            for part in parts.iter().skip(1) {
                if let Some(end) = part.find(')') {
                    let arg_str = &part[..end];
                    println!("{}", arg_str.trim_matches('"'));
                }
            }
        }
        Ok(DvnValue::Shunya)
    }
}

pub struct Samkayati;
impl DhatuFn for Samkayati {
    fn name(&self) -> &'static str {
        "samkayati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "kay"
    }
    fn python_equivalent(&self) -> &'static str {
        "compile"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.len() < 2 {
            return Err(StdlibError::ArgCount {
                dhatu: "samkayati".into(),
                expected: 2,
                got: args.len(),
            });
        }
        let source = match &args[0] {
            DvnValue::Vakya(s) => s,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "samkayati".into(),
                    expected: "Vakya".into(),
                    got: format!("{:?}", args[0]),
                })
            }
        };
        let mode = match &args[1] {
            DvnValue::Vakya(s) => s,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "samkayati".into(),
                    expected: "Vakya".into(),
                    got: format!("{:?}", args[1]),
                })
            }
        };

        if mode == "check" {
            let mut stack = Vec::new();
            for c in source.chars() {
                match c {
                    '(' | '{' | '[' => stack.push(c),
                    ')' => {
                        if stack.pop() != Some('(') {
                            return Err(StdlibError::ConversionError {
                                dhatu: "samkayati".into(),
                                msg: "unbalanced parens".into(),
                            });
                        }
                    }
                    '}' => {
                        if stack.pop() != Some('{') {
                            return Err(StdlibError::ConversionError {
                                dhatu: "samkayati".into(),
                                msg: "unbalanced braces".into(),
                            });
                        }
                    }
                    ']' => {
                        if stack.pop() != Some('[') {
                            return Err(StdlibError::ConversionError {
                                dhatu: "samkayati".into(),
                                msg: "unbalanced brackets".into(),
                            });
                        }
                    }
                    _ => {}
                }
            }
            if !stack.is_empty() {
                return Err(StdlibError::ConversionError {
                    dhatu: "samkayati".into(),
                    msg: "unbalanced symbols".into(),
                });
            }
            Ok(DvnValue::Satya(true))
        } else {
            Err(StdlibError::ConversionError {
                dhatu: "samkayati".into(),
                msg: "unsupported mode".into(),
            })
        }
    }
}

// --- String Dhatus ---

pub struct Samyojayati;
impl DhatuFn for Samyojayati {
    fn name(&self) -> &'static str {
        "samyojayati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "yuj"
    }
    fn python_equivalent(&self) -> &'static str {
        "str.join"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.len() < 2 {
            return Err(StdlibError::ArgCount {
                dhatu: "samyojayati".into(),
                expected: 2,
                got: args.len(),
            });
        }
        let sep = match &args[0] {
            DvnValue::Vakya(s) => s,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "samyojayati".into(),
                    expected: "Vakya".into(),
                    got: format!("{:?}", args[0]),
                })
            }
        };
        let list = match &args[1] {
            DvnValue::Suchi(v) => v,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "samyojayati".into(),
                    expected: "Suchi".into(),
                    got: format!("{:?}", args[1]),
                })
            }
        };

        let mut res = Vec::new();
        for item in list {
            match item {
                DvnValue::Vakya(s) => res.push(s.clone()),
                _ => res.push(format!("{:?}", item)),
            }
        }
        Ok(DvnValue::Vakya(res.join(sep)))
    }
}

pub struct Vibhajati;
impl DhatuFn for Vibhajati {
    fn name(&self) -> &'static str {
        "vibhajati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "bhaj"
    }
    fn python_equivalent(&self) -> &'static str {
        "str.split"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.len() < 2 {
            return Err(StdlibError::ArgCount {
                dhatu: "vibhajati".into(),
                expected: 2,
                got: args.len(),
            });
        }
        let s = match &args[0] {
            DvnValue::Vakya(s) => s,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "vibhajati".into(),
                    expected: "Vakya".into(),
                    got: format!("{:?}", args[0]),
                })
            }
        };
        let sep = match &args[1] {
            DvnValue::Vakya(s) => s,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "vibhajati".into(),
                    expected: "Vakya".into(),
                    got: format!("{:?}", args[1]),
                })
            }
        };

        Ok(DvnValue::Suchi(
            s.split(sep)
                .map(|p| DvnValue::Vakya(p.to_string()))
                .collect(),
        ))
    }
}

pub struct Mudrayati;
impl DhatuFn for Mudrayati {
    fn name(&self) -> &'static str {
        "mudrayati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "mudr"
    }
    fn python_equivalent(&self) -> &'static str {
        "format"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.len() < 2 {
            return Err(StdlibError::ArgCount {
                dhatu: "mudrayati".into(),
                expected: 2,
                got: args.len(),
            });
        }
        let template = match &args[0] {
            DvnValue::Vakya(s) => s,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "mudrayati".into(),
                    expected: "Vakya".into(),
                    got: format!("{:?}", args[0]),
                })
            }
        };
        let values = match &args[1] {
            DvnValue::Suchi(v) => v,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "mudrayati".into(),
                    expected: "Suchi".into(),
                    got: format!("{:?}", args[1]),
                })
            }
        };

        let mut res = template.clone();
        for (i, val) in values.iter().enumerate() {
            let placeholder = format!("{{{}}}", i);
            let val_str = match val {
                DvnValue::Vakya(s) => s.clone(),
                DvnValue::Sankhya(n) => n.to_string(),
                DvnValue::Dasha(f) => f.to_string(),
                _ => format!("{:?}", val),
            };
            res = res.replace(&placeholder, &val_str);
        }
        Ok(DvnValue::Vakya(res))
    }
}

pub struct Shodhatyati;
impl DhatuFn for Shodhatyati {
    fn name(&self) -> &'static str {
        "shodhatyati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "shudh"
    }
    fn python_equivalent(&self) -> &'static str {
        "str.strip"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() {
            return Err(StdlibError::ArgCount {
                dhatu: "shodhatyati".into(),
                expected: 1,
                got: 0,
            });
        }
        let s = match &args[0] {
            DvnValue::Vakya(s) => s,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "shodhatyati".into(),
                    expected: "Vakya".into(),
                    got: format!("{:?}", args[0]),
                })
            }
        };

        let res = if args.len() > 1 {
            if let DvnValue::Vakya(chars) = &args[1] {
                s.trim_matches(|c| chars.contains(c)).to_string()
            } else {
                s.trim().to_string()
            }
        } else {
            s.trim().to_string()
        };
        Ok(DvnValue::Vakya(res))
    }
}

pub struct Uccayati;
impl DhatuFn for Uccayati {
    fn name(&self) -> &'static str {
        "uccayati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "uc"
    }
    fn python_equivalent(&self) -> &'static str {
        "str.upper"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() {
            return Err(StdlibError::ArgCount {
                dhatu: "uccayati".into(),
                expected: 1,
                got: 0,
            });
        }
        let s = match &args[0] {
            DvnValue::Vakya(s) => s,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "uccayati".into(),
                    expected: "Vakya".into(),
                    got: format!("{:?}", args[0]),
                })
            }
        };
        Ok(DvnValue::Vakya(s.to_uppercase()))
    }
}

pub struct Avacayati;
impl DhatuFn for Avacayati {
    fn name(&self) -> &'static str {
        "avacayati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "ava"
    }
    fn python_equivalent(&self) -> &'static str {
        "str.lower"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() {
            return Err(StdlibError::ArgCount {
                dhatu: "avacayati".into(),
                expected: 1,
                got: 0,
            });
        }
        let s = match &args[0] {
            DvnValue::Vakya(s) => s,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "avacayati".into(),
                    expected: "Vakya".into(),
                    got: format!("{:?}", args[0]),
                })
            }
        };
        Ok(DvnValue::Vakya(s.to_lowercase()))
    }
}

pub struct Anveshyati;
impl DhatuFn for Anveshyati {
    fn name(&self) -> &'static str {
        "anveshyati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "ish"
    }
    fn python_equivalent(&self) -> &'static str {
        "str.find"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.len() < 2 {
            return Err(StdlibError::ArgCount {
                dhatu: "anveshyati".into(),
                expected: 2,
                got: args.len(),
            });
        }
        let haystack = match &args[0] {
            DvnValue::Vakya(s) => s,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "anveshyati".into(),
                    expected: "Vakya".into(),
                    got: format!("{:?}", args[0]),
                })
            }
        };
        let needle = match &args[1] {
            DvnValue::Vakya(s) => s,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "anveshyati".into(),
                    expected: "Vakya".into(),
                    got: format!("{:?}", args[1]),
                })
            }
        };

        match haystack.find(needle) {
            Some(i) => Ok(DvnValue::Sankhya(i as i64)),
            None => Ok(DvnValue::Sankhya(-1)),
        }
    }
}

pub struct SthapayatiStr;
impl DhatuFn for SthapayatiStr {
    fn name(&self) -> &'static str {
        "sthapayati_str"
    }
    fn sanskrit_root(&self) -> &'static str {
        "stha"
    }
    fn python_equivalent(&self) -> &'static str {
        "str.replace"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.len() < 3 {
            return Err(StdlibError::ArgCount {
                dhatu: "sthapayati_str".into(),
                expected: 3,
                got: args.len(),
            });
        }
        let s = match &args[0] {
            DvnValue::Vakya(s) => s,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "sthapayati_str".into(),
                    expected: "Vakya".into(),
                    got: format!("{:?}", args[0]),
                })
            }
        };
        let old = match &args[1] {
            DvnValue::Vakya(s) => s,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "sthapayati_str".into(),
                    expected: "Vakya".into(),
                    got: format!("{:?}", args[1]),
                })
            }
        };
        let new = match &args[2] {
            DvnValue::Vakya(s) => s,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "sthapayati_str".into(),
                    expected: "Vakya".into(),
                    got: format!("{:?}", args[2]),
                })
            }
        };

        Ok(DvnValue::Vakya(s.replace(old, new)))
    }
}

// --- Memory + Object ---

pub struct Niveshayati;
impl DhatuFn for Niveshayati {
    fn name(&self) -> &'static str {
        "niveshayati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "vish"
    }
    fn python_equivalent(&self) -> &'static str {
        "import"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() {
            return Err(StdlibError::ArgCount {
                dhatu: "niveshayati".into(),
                expected: 1,
                got: 0,
            });
        }
        let module = match &args[0] {
            DvnValue::Vakya(s) => s,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "niveshayati".into(),
                    expected: "Vakya".into(),
                    got: format!("{:?}", args[0]),
                })
            }
        };

        let supported = [
            "math",
            "io",
            "collections",
            "itertools",
            "object",
            "iteration",
            "introspect",
            "advanced",
        ];
        if supported.contains(&module.as_str()) {
            Ok(DvnValue::Vakya(format!("module:{} loaded", module)))
        } else {
            Err(StdlibError::IoError {
                dhatu: "niveshayati".into(),
                msg: format!("module '{}' not found", module),
            })
        }
    }
}

pub struct Smrtimapati;
impl DhatuFn for Smrtimapati {
    fn name(&self) -> &'static str {
        "smrtimapati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "smr"
    }
    fn python_equivalent(&self) -> &'static str {
        "sizeof"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() {
            return Err(StdlibError::ArgCount {
                dhatu: "smrtimapati".into(),
                expected: 1,
                got: 0,
            });
        }
        let size = match &args[0] {
            DvnValue::Sankhya(_) => 8,
            DvnValue::Dasha(_) => 8,
            DvnValue::Vakya(s) => s.len() as i64,
            DvnValue::Satya(_) => 1,
            DvnValue::Suchi(v) => (v.len() * 8) as i64,
            DvnValue::Sutram(v) => (v.len() * 8) as i64,
            DvnValue::Kosha(m) => (m.len() * 16) as i64,
            DvnValue::Samuha(s) => (s.len() * 8) as i64,
            DvnValue::Kriya(s) => s.len() as i64,
            DvnValue::Shunya => 0,
        };
        Ok(DvnValue::Sankhya(size))
    }
}

pub struct Anukramati;
impl DhatuFn for Anukramati {
    fn name(&self) -> &'static str {
        "anukramati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "kram"
    }
    fn python_equivalent(&self) -> &'static str {
        "slice"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.len() < 3 {
            return Err(StdlibError::ArgCount {
                dhatu: "anukramati".into(),
                expected: 3,
                got: args.len(),
            });
        }
        let list = match &args[0] {
            DvnValue::Suchi(v) => v,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "anukramati".into(),
                    expected: "Suchi".into(),
                    got: format!("{:?}", args[0]),
                })
            }
        };
        let start = match args[1] {
            DvnValue::Sankhya(n) => n as usize,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "anukramati".into(),
                    expected: "Sankhya".into(),
                    got: format!("{:?}", args[1]),
                })
            }
        };
        let stop = match args[2] {
            DvnValue::Sankhya(n) => n as usize,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "anukramati".into(),
                    expected: "Sankhya".into(),
                    got: format!("{:?}", args[2]),
                })
            }
        };
        let step = match args.get(3) {
            Some(DvnValue::Sankhya(n)) => *n as usize,
            _ => 1,
        };

        let mut res = Vec::new();
        let mut curr = start;
        while curr < stop && curr < list.len() {
            res.push(list[curr].clone());
            curr += step;
        }
        Ok(DvnValue::Suchi(res))
    }
}

pub struct Sthirikarati;
impl DhatuFn for Sthirikarati {
    fn name(&self) -> &'static str {
        "sthirikarati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "stha"
    }
    fn python_equivalent(&self) -> &'static str {
        "frozenset"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() {
            return Err(StdlibError::ArgCount {
                dhatu: "sthirikarati".into(),
                expected: 1,
                got: 0,
            });
        }
        let list = match &args[0] {
            DvnValue::Suchi(v) => v,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "sthirikarati".into(),
                    expected: "Suchi".into(),
                    got: format!("{:?}", args[0]),
                })
            }
        };

        let mut set: Vec<String> = list
            .iter()
            .map(|item| match item {
                DvnValue::Vakya(s) => s.clone(),
                _ => format!("{:?}", item),
            })
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        set.sort();

        Ok(DvnValue::Sutram(
            set.into_iter().map(DvnValue::Vakya).collect(),
        ))
    }
}

// --- Functional ---

pub struct YojayatiFn;
impl DhatuFn for YojayatiFn {
    fn name(&self) -> &'static str {
        "yojayati_fn"
    }
    fn sanskrit_root(&self) -> &'static str {
        "yuj"
    }
    fn python_equivalent(&self) -> &'static str {
        "reduce"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.len() < 2 {
            return Err(StdlibError::ArgCount {
                dhatu: "yojayati_fn".into(),
                expected: 2,
                got: args.len(),
            });
        }
        let fn_name = match &args[0] {
            DvnValue::Kriya(s) => s,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "yojayati_fn".into(),
                    expected: "Kriya".into(),
                    got: format!("{:?}", args[0]),
                })
            }
        };
        let list = match &args[1] {
            DvnValue::Suchi(v) => v,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "yojayati_fn".into(),
                    expected: "Suchi".into(),
                    got: format!("{:?}", args[1]),
                })
            }
        };

        if list.is_empty() {
            return Ok(DvnValue::Shunya);
        }

        let registry = crate::registry::DhatuRegistry::new();
        let mut acc = list[0].clone();
        for item in list.iter().skip(1) {
            acc = registry.call(fn_name, vec![acc, item.clone()])?;
        }
        Ok(acc)
    }
}

pub struct Vargiyati;
impl DhatuFn for Vargiyati {
    fn name(&self) -> &'static str {
        "vargiyati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "varg"
    }
    fn python_equivalent(&self) -> &'static str {
        "sorted_key"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.len() < 2 {
            return Err(StdlibError::ArgCount {
                dhatu: "vargiyati".into(),
                expected: 2,
                got: args.len(),
            });
        }
        let list = match &args[0] {
            DvnValue::Suchi(v) => v.clone(),
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "vargiyati".into(),
                    expected: "Suchi".into(),
                    got: format!("{:?}", args[0]),
                })
            }
        };
        let key_fn = match &args[1] {
            DvnValue::Vakya(s) => s,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "vargiyati".into(),
                    expected: "Vakya (key fn name)".into(),
                    got: format!("{:?}", args[1]),
                })
            }
        };

        let registry = crate::registry::DhatuRegistry::new();
        let mut keyed: Vec<(DvnValue, DvnValue)> = Vec::new();
        for item in list {
            let key = registry.call(key_fn, vec![item.clone()])?;
            keyed.push((key, item));
        }

        keyed.sort_by(|a, b| match (&a.0, &b.0) {
            (DvnValue::Sankhya(x), DvnValue::Sankhya(y)) => x.cmp(y),
            (DvnValue::Vakya(x), DvnValue::Vakya(y)) => x.cmp(y),
            _ => std::cmp::Ordering::Equal,
        });

        Ok(DvnValue::Suchi(keyed.into_iter().map(|(_, v)| v).collect()))
    }
}

// --- I/O Extended ---

pub struct Samcayati;
impl DhatuFn for Samcayati {
    fn name(&self) -> &'static str {
        "samcayati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "ci"
    }
    fn python_equivalent(&self) -> &'static str {
        "print_end"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.len() < 2 {
            return Err(StdlibError::ArgCount {
                dhatu: "samcayati".into(),
                expected: 2,
                got: args.len(),
            });
        }
        let values = match &args[0] {
            DvnValue::Suchi(v) => v,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "samcayati".into(),
                    expected: "Suchi".into(),
                    got: format!("{:?}", args[0]),
                })
            }
        };
        let end = match &args[1] {
            DvnValue::Vakya(s) => s,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "samcayati".into(),
                    expected: "Vakya".into(),
                    got: format!("{:?}", args[1]),
                })
            }
        };

        for val in values {
            match val {
                DvnValue::Vakya(s) => print!("{}", s),
                _ => print!("{:?}", val),
            }
        }
        print!("{}", end);
        use std::io::Write;
        std::io::stdout()
            .flush()
            .map_err(|e| StdlibError::IoError {
                dhatu: "samcayati".into(),
                msg: e.to_string(),
            })?;
        Ok(DvnValue::Shunya)
    }
}

pub struct Parivartati;
impl DhatuFn for Parivartati {
    fn name(&self) -> &'static str {
        "parivartati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "vrt"
    }
    fn python_equivalent(&self) -> &'static str {
        "input_cast"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.len() < 2 {
            return Err(StdlibError::ArgCount {
                dhatu: "parivartati".into(),
                expected: 2,
                got: args.len(),
            });
        }
        let prompt = match &args[0] {
            DvnValue::Vakya(s) => s,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "parivartati".into(),
                    expected: "Vakya".into(),
                    got: format!("{:?}", args[0]),
                })
            }
        };
        let type_name = match &args[1] {
            DvnValue::Vakya(s) => s,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "parivartati".into(),
                    expected: "Vakya".into(),
                    got: format!("{:?}", args[1]),
                })
            }
        };

        print!("{}", prompt);
        use std::io::Write;
        std::io::stdout()
            .flush()
            .map_err(|e| StdlibError::IoError {
                dhatu: "parivartati".into(),
                msg: e.to_string(),
            })?;

        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(|e| StdlibError::IoError {
                dhatu: "parivartati".into(),
                msg: e.to_string(),
            })?;
        let input = input.trim();

        match type_name.as_str() {
            "Sankhya" => input.parse::<i64>().map(DvnValue::Sankhya).map_err(|e| {
                StdlibError::ConversionError {
                    dhatu: "parivartati".into(),
                    msg: e.to_string(),
                }
            }),
            "Dasha" => input.parse::<f64>().map(DvnValue::Dasha).map_err(|e| {
                StdlibError::ConversionError {
                    dhatu: "parivartati".into(),
                    msg: e.to_string(),
                }
            }),
            "Vakya" => Ok(DvnValue::Vakya(input.to_string())),
            _ => Err(StdlibError::TypeError {
                dhatu: "parivartati".into(),
                expected: "valid type name".into(),
                got: type_name.clone(),
            }),
        }
    }
}

// --- Meta ---

pub struct Pratinidhayati;
impl DhatuFn for Pratinidhayati {
    fn name(&self) -> &'static str {
        "pratinidhayati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "dha"
    }
    fn python_equivalent(&self) -> &'static str {
        "repr_pretty"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() {
            return Err(StdlibError::ArgCount {
                dhatu: "pratinidhayati".into(),
                expected: 1,
                got: 0,
            });
        }
        Ok(DvnValue::Vakya(format!("{:#?}", args[0])))
    }
}

pub struct Mulayati;
impl DhatuFn for Mulayati {
    fn name(&self) -> &'static str {
        "mulayati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "mul"
    }
    fn python_equivalent(&self) -> &'static str {
        "object"
    }
    fn call(&self, _args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        Ok(DvnValue::Kosha(HashMap::new()))
    }
}
