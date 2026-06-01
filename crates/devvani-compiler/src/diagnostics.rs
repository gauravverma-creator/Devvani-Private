use devvani_typesystem::TypeCheckError;
use devvani_codegen::CodegenError;
use crate::CompilerError;

// Severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Dosha,    // Error   (दोष)
    Sanka,    // Warning (शंका)
    Suchana,  // Info    (सूचना)
}

// A single diagnostic message
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,          // e.g. "D001", "S002"
    pub sanskrit_title: String,// e.g. "अपरिचित नाम"
    pub roman_title: String,   // e.g. "Aparicita Nama"
    pub message: String,       // full explanation
    pub sutra_ref: Option<String>, // e.g. "sutra 1.4.54"
    pub hint: Option<String>,  // suggested fix
}

impl Diagnostic {
    pub fn display(&self) -> String {
        let severity_str = match self.severity {
            Severity::Dosha => "दोष Dosha",
            Severity::Sanka => "शंका Sanka",
            Severity::Suchana => "सूचना Suchana",
        };

        let mut output = format!(
            "── {} {} | {} ──────────────────\n",
            severity_str, self.code, self.roman_title
        );
        output.push_str(&format!(" {}: {}\n", self.sanskrit_title, self.message));
        
        if let Some(sutra) = &self.sutra_ref {
            output.push_str(&format!(" Sutra: {}\n", sutra));
        }
        
        if let Some(hint) = &self.hint {
            output.push_str(&format!(" Hint: {}\n", hint));
        }
        
        output.push_str("────────────────────────────────────────────────");
        output
    }
}

// Diagnostic registry — maps error kinds to Diagnostics
pub struct DiagnosticEngine;

impl DiagnosticEngine {
    pub fn from_type_error(err: &TypeCheckError) -> Diagnostic {
        match err {
            TypeCheckError::UndefinedName(name) => Diagnostic {
                severity: Severity::Dosha,
                code: "D001".to_string(),
                sanskrit_title: "अपरिचित नाम".to_string(),
                roman_title: "Aparicita Nama".to_string(),
                message: format!(
                    "'{}' Prathama Vibhakti mein Kartā ke roop mein \
                     nahi mila. Pehle ise define karo.", name),
                sutra_ref: Some("1.4.54 (Kartā — svatantraḥ kartā)".to_string()),
                hint: Some(format!("'{}' ko pehle declare karo: rāmaḥ", name)),
            },
            TypeCheckError::TypeMismatch { expected, found } => Diagnostic {
                severity: Severity::Dosha,
                code: "D002".to_string(),
                sanskrit_title: "विभक्ति-भेद".to_string(),
                roman_title: "Vibhakti Bheda".to_string(),
                message: format!(
                    "Pratyāśit (expected): {} — Prāpta (found): {}. \
                     Vibhakti mismatch.", expected, found),
                sutra_ref: Some("1.1.2".to_string()),
                hint: Some("Sahi Vibhakti pratyaya lagao.".to_string()),
            },
            TypeCheckError::InvalidVibhaktiUsage(msg) => Diagnostic {
                severity: Severity::Dosha,
                code: "D003".to_string(),
                sanskrit_title: "अशुद्ध विभक्ति".to_string(),
                roman_title: "Ashuddha Vibhakti".to_string(),
                message: format!("Vibhakti ka galat prayog: {}", msg),
                sutra_ref: Some("2.3.1".to_string()),
                hint: Some("Karaka aur Vibhakti ka mel check karo.".to_string()),
            },
        }
    }

    pub fn from_codegen_error(err: &CodegenError) -> Diagnostic {
        match err {
            CodegenError::UnsupportedNode(n) => Diagnostic {
                severity: Severity::Dosha,
                code: "D004".to_string(),
                sanskrit_title: "असमर्थित पद".to_string(),
                roman_title: "Asamarthita Pada".to_string(),
                message: format!("'{}' — yeh pada abhi codegen mein \
                                  samarthit nahi.", n),
                sutra_ref: None,
                hint: Some("Devvani ke supported constructs dekho.".to_string()),
            },
            CodegenError::TypeCheckFailed(msg) => Diagnostic {
                severity: Severity::Dosha,
                code: "D005".to_string(),
                sanskrit_title: "प्रकार-परीक्षा विफल".to_string(),
                roman_title: "Prakar Pariksha Vifal".to_string(),
                message: format!("Type check fail: {}", msg),
                sutra_ref: None,
                hint: None,
            },
            CodegenError::IoError(msg) => Diagnostic {
                severity: Severity::Dosha,
                code: "D006".to_string(),
                sanskrit_title: "संचिका-दोष".to_string(),
                roman_title: "Sanchika Dosha".to_string(),
                message: format!("File operation fail: {}", msg),
                sutra_ref: None,
                hint: Some("File path aur permissions check karo.".to_string()),
            },
        }
    }

    pub fn from_compiler_error(err: &CompilerError) -> Diagnostic {
        match err {
            CompilerError::IoError(msg) => Diagnostic {
                severity: Severity::Dosha,
                code: "D007".to_string(),
                sanskrit_title: "संचिका-दोष".to_string(),
                roman_title: "Sanchika Dosha".to_string(),
                message: format!("'{}' file nahi mili ya padhi nahi ja \
                                  sakti.", msg),
                sutra_ref: None,
                hint: Some("Sahi file path do: devvani compile \
                            <file.dvn>".to_string()),
            },
            CompilerError::LexError(msg) => Diagnostic {
                severity: Severity::Dosha,
                code: "D008".to_string(),
                sanskrit_title: "वर्ण-विश्लेषण-दोष".to_string(),
                roman_title: "Varna Vishleshan Dosha".to_string(),
                message: format!("Shabda pahchana mein samasya: {}", msg),
                sutra_ref: Some("1.1.1".to_string()),
                hint: Some("IAST Unicode sahi hai? \
                            Matra aur anusvara check karo.".to_string()),
            },
            CompilerError::ParseError(msg) => Diagnostic {
                severity: Severity::Dosha,
                code: "D009".to_string(),
                sanskrit_title: "वाक्य-संरचना-दोष".to_string(),
                roman_title: "Vakya Sanrachna Dosha".to_string(),
                message: format!("SOV krama galat hai: {}", msg),
                sutra_ref: Some("2.1.1".to_string()),
                hint: Some("Devvani SOV order follow karo: \
                            Kartā Karma Kriyā.".to_string()),
            },
            CompilerError::CodegenError(msg) => Diagnostic {
                severity: Severity::Dosha,
                code: "D010".to_string(),
                sanskrit_title: "कोड-निर्माण-दोष".to_string(),
                roman_title: "Code Nirman Dosha".to_string(),
                message: format!("Rust code generation mein samasya: \
                                  {}", msg),
                sutra_ref: None,
                hint: None,
            },
        }
    }

    pub fn report(diagnostics: &[Diagnostic]) -> String {
        if diagnostics.is_empty() {
            return "✓ Shuddham — कोई दोष नही | No errors found.\n"
                .to_string();
        }
        diagnostics.iter()
            .map(|d| d.display())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use devvani_typesystem::TypeCheckError;

    #[test]
    fn test_from_type_error_undefined() {
        let err = TypeCheckError::UndefinedName("ramah".to_string());
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D001");
        assert!(diag.display().contains("Aparicita Nama"));
    }

    #[test]
    fn test_from_compiler_error_parse() {
        let err = CompilerError::ParseError("test".to_string());
        let diag = DiagnosticEngine::from_compiler_error(&err);
        assert_eq!(diag.code, "D009");
        assert!(diag.display().contains("SOV"));
    }

    #[test]
    fn test_report_empty() {
        let report = DiagnosticEngine::report(&[]);
        assert!(report.contains("Shuddham"));
    }

    #[test]
    fn test_report_with_diagnostics() {
        let err1 = TypeCheckError::UndefinedName("ramah".to_string());
        let diag1 = DiagnosticEngine::from_type_error(&err1);
        let err2 = CompilerError::ParseError("test".to_string());
        let diag2 = DiagnosticEngine::from_compiler_error(&err2);
        
        let report = DiagnosticEngine::report(&[diag1, diag2]);
        assert!(report.contains("D001"));
        assert!(report.contains("D009"));
    }
}
