use crate::{DhatuFn, DvnValue, StdlibError};

pub struct Samgrhnati;
impl DhatuFn for Samgrhnati {
    fn name(&self) -> &'static str {
        "samgrhnati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "grah"
    }
    fn python_equivalent(&self) -> &'static str {
        "vars"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() {
            return Err(StdlibError::ArgCount {
                dhatu: "samgrhnati".into(),
                expected: 1,
                got: 0,
            });
        }
        match &args[0] {
            DvnValue::Kosha(_) => Ok(args[0].clone()),
            _ => Err(StdlibError::TypeError {
                dhatu: "samgrhnati".into(),
                expected: "Kosha".into(),
                got: format!("{:?}", args[0]),
            }),
        }
    }
}

pub struct Nirdisati;
impl DhatuFn for Nirdisati {
    fn name(&self) -> &'static str {
        "nirdisati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "dis"
    }
    fn python_equivalent(&self) -> &'static str {
        "dir"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() {
            return Err(StdlibError::ArgCount {
                dhatu: "nirdisati".into(),
                expected: 1,
                got: 0,
            });
        }
        let methods = match &args[0] {
            DvnValue::Sankhya(_) => vec!["tulayati", "vardhayati", "parinameti", "dvibhajati"],
            DvnValue::Suchi(_) => vec!["ganavati", "kramayati", "viparitayati", "yojayati"],
            _ => vec!["janayati", "prakashati", "mapayati"],
        };
        Ok(DvnValue::Suchi(
            methods
                .into_iter()
                .map(|s| DvnValue::Vakya(s.into()))
                .collect(),
        ))
    }
}

pub struct Darsayati;
impl DhatuFn for Darsayati {
    fn name(&self) -> &'static str {
        "darsayati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "drs"
    }
    fn python_equivalent(&self) -> &'static str {
        "help"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() {
            return Err(StdlibError::ArgCount {
                dhatu: "darsayati".into(),
                expected: 1,
                got: 0,
            });
        }
        let name = match &args[0] {
            DvnValue::Vakya(s) => s,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "darsayati".into(),
                    expected: "Vakya (dhatu name)".into(),
                    got: format!("{:?}", args[0]),
                })
            }
        };
        // This is a bit recursive since we can't easily access the registry from here without passing it.
        // For now, return a generic help string or handle a few common ones.
        Ok(DvnValue::Vakya(format!("{}: Devvani built-in dhatu", name)))
    }
}

pub struct Mulyayati;
impl DhatuFn for Mulyayati {
    fn name(&self) -> &'static str {
        "mulyayati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "mul"
    }
    fn python_equivalent(&self) -> &'static str {
        "eval"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() {
            return Err(StdlibError::ArgCount {
                dhatu: "mulyayati".into(),
                expected: 1,
                got: 0,
            });
        }
        let expr = match &args[0] {
            DvnValue::Vakya(s) => s,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "mulyayati".into(),
                    expected: "Vakya (expression)".into(),
                    got: format!("{:?}", args[0]),
                })
            }
        };

        // Simple arithmetic evaluator for "x + y" or "x * y"
        let parts: Vec<&str> = expr.split_whitespace().collect();
        if parts.len() == 3 {
            let a = parts[0]
                .parse::<i64>()
                .map_err(|e| StdlibError::ConversionError {
                    dhatu: "mulyayati".into(),
                    msg: e.to_string(),
                })?;
            let b = parts[2]
                .parse::<i64>()
                .map_err(|e| StdlibError::ConversionError {
                    dhatu: "mulyayati".into(),
                    msg: e.to_string(),
                })?;
            let res = match parts[1] {
                "+" => a + b,
                "-" => a - b,
                "*" => a * b,
                "/" => {
                    if b != 0 {
                        a / b
                    } else {
                        return Err(StdlibError::IoError {
                            dhatu: "mulyayati".into(),
                            msg: "div by zero".into(),
                        });
                    }
                }
                _ => {
                    return Err(StdlibError::ConversionError {
                        dhatu: "mulyayati".into(),
                        msg: "unsupported operator".into(),
                    })
                }
            };
            Ok(DvnValue::Sankhya(res))
        } else {
            Err(StdlibError::ConversionError {
                dhatu: "mulyayati".into(),
                msg: "unsupported expression format".into(),
            })
        }
    }
}
