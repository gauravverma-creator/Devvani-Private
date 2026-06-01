#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SandhiMode {
    Auto,
    Off,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SandhiLog {
    pub rule: String,
    pub original: String,
    pub replacement: String,
    pub pos: usize,
}

pub struct SandhiEngine {
    pub mode: SandhiMode,
    pub logs: Vec<SandhiLog>,
}

impl SandhiEngine {
    pub fn new(mode: SandhiMode) -> Self {
        Self {
            mode,
            logs: Vec::new(),
        }
    }

    pub fn apply(&mut self, input: &str) -> String {
        if self.mode == SandhiMode::Off {
            return input.to_string();
        }

        let mut output = input.to_string();
        
        // Rule 1: Savarna Dirgha
        self.apply_replacement(&mut output, "a+a", "ā", "Savarna Dirgha");
        self.apply_replacement(&mut output, "i+i", "ī", "Savarna Dirgha");
        self.apply_replacement(&mut output, "u+u", "ū", "Savarna Dirgha");

        // Rule 2: Guna
        self.apply_replacement(&mut output, "a+i", "e", "Guna");
        self.apply_replacement(&mut output, "a+u", "o", "Guna");

        // Rule 3: Vriddhi
        self.apply_replacement(&mut output, "ā+i", "ai", "Vriddhi");
        self.apply_replacement(&mut output, "ā+u", "au", "Vriddhi");

        // Rule 4: Yan Sandhi
        self.apply_replacement(&mut output, "i+a", "ya", "Yan Sandhi");
        self.apply_replacement(&mut output, "u+a", "va", "Yan Sandhi");

        // Rule 5: Visarga Sandhi (Simplified)
        self.apply_replacement(&mut output, "ḥ+c", "śc", "Visarga Sandhi");
        self.apply_replacement(&mut output, "ḥ+t", "st", "Visarga Sandhi");
        
        output
    }

    fn apply_replacement(&mut self, s: &mut String, pattern: &str, replacement: &str, rule_name: &str) {
        let mut start = 0;
        while let Some(pos) = s[start..].find(pattern) {
            let actual_pos = start + pos;
            self.logs.push(SandhiLog {
                rule: rule_name.to_string(),
                original: pattern.to_string(),
                replacement: replacement.to_string(),
                pos: actual_pos,
            });
            s.replace_range(actual_pos..actual_pos + pattern.len(), replacement);
            start = actual_pos + replacement.len();
        }
    }
}
