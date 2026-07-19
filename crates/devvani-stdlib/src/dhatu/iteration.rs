use crate::{DhatuFn, DvnValue, StdlibError};

pub struct Avahayati;
impl DhatuFn for Avahayati {
    fn name(&self) -> &'static str {
        "avahayati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "vah"
    }
    fn python_equivalent(&self) -> &'static str {
        "iter"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() {
            return Err(StdlibError::ArgCount {
                dhatu: "avahayati".into(),
                expected: 1,
                got: 0,
            });
        }
        match &args[0] {
            DvnValue::Suchi(_)
            | DvnValue::Sutram(_)
            | DvnValue::Vakya(_)
            | DvnValue::Kosha(_)
            | DvnValue::Samuha(_) => Ok(args[0].clone()),
            _ => Err(StdlibError::TypeError {
                dhatu: "avahayati".into(),
                expected: "iterable".into(),
                got: format!("{:?}", args[0]),
            }),
        }
    }
}

pub struct Agrayati;
impl DhatuFn for Agrayati {
    fn name(&self) -> &'static str {
        "agrayati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "agra"
    }
    fn python_equivalent(&self) -> &'static str {
        "next"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.len() < 2 {
            return Err(StdlibError::ArgCount {
                dhatu: "agrayati".into(),
                expected: 2,
                got: args.len(),
            });
        }
        let index = match args[1] {
            DvnValue::Sankhya(n) => n as usize,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "agrayati".into(),
                    expected: "Sankhya index".into(),
                    got: format!("{:?}", args[1]),
                })
            }
        };

        match &args[0] {
            DvnValue::Suchi(v) | DvnValue::Sutram(v) => {
                if index < v.len() {
                    Ok(DvnValue::Sutram(vec![
                        v[index].clone(),
                        DvnValue::Sankhya((index + 1) as i64),
                    ]))
                } else {
                    Ok(DvnValue::Shunya)
                }
            }
            DvnValue::Vakya(s) => {
                let chars: Vec<char> = s.chars().collect();
                if index < chars.len() {
                    Ok(DvnValue::Sutram(vec![
                        DvnValue::Vakya(chars[index].to_string()),
                        DvnValue::Sankhya((index + 1) as i64),
                    ]))
                } else {
                    Ok(DvnValue::Shunya)
                }
            }
            _ => Err(StdlibError::TypeError {
                dhatu: "agrayati".into(),
                expected: "sequence".into(),
                got: format!("{:?}", args[0]),
            }),
        }
    }
}

pub struct Sarvayati;
impl DhatuFn for Sarvayati {
    fn name(&self) -> &'static str {
        "sarvayati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "sarva"
    }
    fn python_equivalent(&self) -> &'static str {
        "all"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() {
            return Err(StdlibError::ArgCount {
                dhatu: "sarvayati".into(),
                expected: 1,
                got: 0,
            });
        }
        let items = match &args[0] {
            DvnValue::Suchi(v) | DvnValue::Sutram(v) => v,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "sarvayati".into(),
                    expected: "sequence".into(),
                    got: format!("{:?}", args[0]),
                })
            }
        };
        for item in items {
            let truthy = match item {
                DvnValue::Satya(b) => *b,
                DvnValue::Sankhya(n) => *n != 0,
                DvnValue::Dasha(f) => *f != 0.0,
                DvnValue::Vakya(s) => !s.is_empty(),
                DvnValue::Shunya => false,
                _ => true,
            };
            if !truthy {
                return Ok(DvnValue::Satya(false));
            }
        }
        Ok(DvnValue::Satya(true))
    }
}

pub struct Ekayati;
impl DhatuFn for Ekayati {
    fn name(&self) -> &'static str {
        "ekayati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "eka"
    }
    fn python_equivalent(&self) -> &'static str {
        "any"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() {
            return Err(StdlibError::ArgCount {
                dhatu: "ekayati".into(),
                expected: 1,
                got: 0,
            });
        }
        let items = match &args[0] {
            DvnValue::Suchi(v) | DvnValue::Sutram(v) => v,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "ekayati".into(),
                    expected: "sequence".into(),
                    got: format!("{:?}", args[0]),
                })
            }
        };
        for item in items {
            let truthy = match item {
                DvnValue::Satya(b) => *b,
                DvnValue::Sankhya(n) => *n != 0,
                DvnValue::Dasha(f) => *f != 0.0,
                DvnValue::Vakya(s) => !s.is_empty(),
                DvnValue::Shunya => false,
                _ => true,
            };
            if truthy {
                return Ok(DvnValue::Satya(true));
            }
        }
        Ok(DvnValue::Satya(false))
    }
}
