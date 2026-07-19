use crate::{DhatuFn, DvnValue, StdlibError};
use std::fs;
use std::io::{self, Write};

pub struct Vadati;
impl DhatuFn for Vadati {
    fn name(&self) -> &'static str {
        "vadati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "vad"
    }
    fn python_equivalent(&self) -> &'static str {
        "print"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        let output = args
            .iter()
            .map(|arg| match arg {
                DvnValue::Vakya(s) => s.clone(),
                DvnValue::Sankhya(n) => n.to_string(),
                DvnValue::Dasha(f) => f.to_string(),
                DvnValue::Satya(b) => b.to_string(),
                _ => format!("{:?}", arg),
            })
            .collect::<Vec<String>>()
            .join(" ");
        println!("{}", output);
        Ok(DvnValue::Shunya)
    }
}

pub struct Pathati;
impl DhatuFn for Pathati {
    fn name(&self) -> &'static str {
        "pathati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "path"
    }
    fn python_equivalent(&self) -> &'static str {
        "input"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if !args.is_empty() {
            if let DvnValue::Vakya(prompt) = &args[0] {
                print!("{}", prompt);
                io::stdout().flush().map_err(|e| StdlibError::IoError {
                    dhatu: "pathati".into(),
                    msg: e.to_string(),
                })?;
            }
        }
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| StdlibError::IoError {
                dhatu: "pathati".into(),
                msg: e.to_string(),
            })?;
        Ok(DvnValue::Vakya(input.trim_end().to_string()))
    }
}

pub struct Likhati;
impl DhatuFn for Likhati {
    fn name(&self) -> &'static str {
        "likhati"
    }
    fn sanskrit_root(&self) -> &'static str {
        "likh"
    }
    fn python_equivalent(&self) -> &'static str {
        "write"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.len() < 2 {
            return Err(StdlibError::ArgCount {
                dhatu: "likhati".into(),
                expected: 2,
                got: args.len(),
            });
        }
        let filename = match &args[0] {
            DvnValue::Vakya(s) => s,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "likhati".into(),
                    expected: "Vakya".into(),
                    got: format!("{:?}", args[0]),
                })
            }
        };
        let content = match &args[1] {
            DvnValue::Vakya(s) => s,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "likhati".into(),
                    expected: "Vakya".into(),
                    got: format!("{:?}", args[1]),
                })
            }
        };
        fs::write(filename, content).map_err(|e| StdlibError::IoError {
            dhatu: "likhati".into(),
            msg: e.to_string(),
        })?;
        Ok(DvnValue::Satya(true))
    }
}

pub struct PathatiFile;
impl DhatuFn for PathatiFile {
    fn name(&self) -> &'static str {
        "pathati_file"
    }
    fn sanskrit_root(&self) -> &'static str {
        "path"
    }
    fn python_equivalent(&self) -> &'static str {
        "read_file"
    }
    fn call(&self, args: Vec<DvnValue>) -> Result<DvnValue, StdlibError> {
        if args.is_empty() {
            return Err(StdlibError::ArgCount {
                dhatu: "pathati_file".into(),
                expected: 1,
                got: 0,
            });
        }
        let filename = match &args[0] {
            DvnValue::Vakya(s) => s,
            _ => {
                return Err(StdlibError::TypeError {
                    dhatu: "pathati_file".into(),
                    expected: "Vakya".into(),
                    got: format!("{:?}", args[0]),
                })
            }
        };
        let content = fs::read_to_string(filename).map_err(|e| StdlibError::IoError {
            dhatu: "pathati_file".into(),
            msg: e.to_string(),
        })?;
        Ok(DvnValue::Vakya(content))
    }
}
