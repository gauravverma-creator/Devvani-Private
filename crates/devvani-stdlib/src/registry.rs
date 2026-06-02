use crate::{DvnValue, DhatuFn, StdlibError};
use std::collections::HashMap;

pub struct DhatuRegistry {
    dhatus: HashMap<&'static str, Box<dyn DhatuFn>>,
}

impl DhatuRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            dhatus: HashMap::new(),
        };
        registry.register_all();
        registry
    }

    fn register_all(&mut self) {
        use crate::dhatu::{io::*, types::*, collections::*, itertools::*, object::*, iteration::*, math::*, introspect::*, advanced::*};
        
        // Part 1
        self.register(Box::new(Vadati));
        self.register(Box::new(Pathati));
        self.register(Box::new(Likhati));
        self.register(Box::new(PathatiFile));
        
        self.register(Box::new(PariNameti));
        self.register(Box::new(Vakyayati));
        self.register(Box::new(Kampayati));
        self.register(Box::new(Dvayati));
        self.register(Box::new(PrakaShati));
        self.register(Box::new(Manayati));
        
        self.register(Box::new(Sucayati));
        self.register(Box::new(Sutrayati));
        self.register(Box::new(Koshayati));
        self.register(Box::new(Sthapayati));
        
        self.register(Box::new(Kramate));
        self.register(Box::new(Yugmayati));
        self.register(Box::new(Citrayati));
        self.register(Box::new(Chinati));
        self.register(Box::new(Kramankati));
        self.register(Box::new(Uttamayati));
        self.register(Box::new(Avamayati));
        self.register(Box::new(Yojayati));
        self.register(Box::new(Kramayati));
        self.register(Box::new(Viparitayati));
        self.register(Box::new(Ganavati));

        // Part 2
        self.register(Box::new(Asti));
        self.register(Box::new(Vidyate));
        self.register(Box::new(Grhnati));
        self.register(Box::new(SthapayatiAttr));
        self.register(Box::new(Janayati));
        self.register(Box::new(Avatarayati));
        self.register(Box::new(Mapayati));
        self.register(Box::new(Kalpayati));

        self.register(Box::new(Avahayati));
        self.register(Box::new(Agrayati));
        self.register(Box::new(Sarvayati));
        self.register(Box::new(Ekayati));

        self.register(Box::new(Tulayati));
        self.register(Box::new(Seshayati));
        self.register(Box::new(Vardhayati));
        self.register(Box::new(Purnayati));
        self.register(Box::new(Dvibhajati));
        self.register(Box::new(Astabhajati));
        self.register(Box::new(Sodasabhajati));
        self.register(Box::new(Varnayati));
        self.register(Box::new(Samkhyati));

        self.register(Box::new(Samgrhnati));
        self.register(Box::new(Nirdisati));
        self.register(Box::new(Darsayati));
        self.register(Box::new(Mulyayati));

        // Part 3
        self.register(Box::new(Arabhati));
        self.register(Box::new(Samkayati));
        self.register(Box::new(Samyojayati));
        self.register(Box::new(Vibhajati));
        self.register(Box::new(Mudrayati));
        self.register(Box::new(Shodhatyati));
        self.register(Box::new(Uccayati));
        self.register(Box::new(Avacayati));
        self.register(Box::new(Anveshyati));
        self.register(Box::new(SthapayatiStr));
        self.register(Box::new(Niveshayati));
        self.register(Box::new(Smrtimapati));
        self.register(Box::new(Anukramati));
        self.register(Box::new(Sthirikarati));
        self.register(Box::new(YojayatiFn));
        self.register(Box::new(Vargiyati));
        self.register(Box::new(Samcayati));
        self.register(Box::new(Parivartati));
        self.register(Box::new(Pratinidhayati));
        self.register(Box::new(Mulayati));
    }

    pub fn register(&mut self, dhatu: Box<dyn DhatuFn>) {
        self.dhatus.insert(dhatu.name(), dhatu);
    }

    pub fn get(&self, name: &str) -> Option<&dyn DhatuFn> {
        self.dhatus.get(name).map(|d| d.as_ref())
    }

    pub fn call(&self, name: &str, args: Vec<DvnValue>) 
        -> Result<DvnValue, StdlibError> 
    {
        self.get(name)
            .ok_or_else(|| StdlibError::TypeError {
                dhatu: name.to_string(),
                expected: "known dhatu".to_string(),
                got: "unknown".to_string(),
            })?
            .call(args)
    }

    pub fn list_all(&self) -> Vec<&'static str> {
        self.dhatus.keys().copied().collect()
    }
}
