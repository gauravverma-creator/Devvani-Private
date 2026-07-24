use crate::CompilerError;
use devvani_codegen::CodegenError;
use devvani_typesystem::TypeCheckError;

// Severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Dosha,   // Error   (दोष)
    Sanka,   // Warning (शंका)
    Suchana, // Info    (सूचना)
}

// A single diagnostic message
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,              // e.g. "D001", "S002"
    pub sanskrit_title: String,    // e.g. "अपरिचित नाम"
    pub roman_title: String,       // e.g. "Aparicita Nama"
    pub message: String,           // full explanation
    pub sutra_ref: Option<String>, // e.g. "sutra 1.4.54"
    pub hint: Option<String>,      // suggested fix
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
            TypeCheckError::NaamaApraapta(name) => Diagnostic {
                severity: Severity::Dosha,
                code: "D001".to_string(),
                sanskrit_title: "अपरिचित नाम".to_string(),
                roman_title: "Aparicita Nama".to_string(),
                message: format!(
                    "'{}' Prathama Vibhakti mein Kartā ke roop mein \
                     nahi mila. Pehle ise define karo.",
                    name
                ),
                sutra_ref: Some("1.4.54 (Kartā — svatantraḥ kartā)".to_string()),
                hint: Some(format!("'{}' ko pehle declare karo: rāmaḥ", name)),
            },
            TypeCheckError::PrakaaraVaisamya { expected, found } => Diagnostic {
                severity: Severity::Dosha,
                code: "D002".to_string(),
                sanskrit_title: "विभक्ति-भेद".to_string(),
                roman_title: "Vibhakti Bheda".to_string(),
                message: format!(
                    "Pratyāśit (expected): {} — Prāpta (found): {}. \
                     Vibhakti mismatch.",
                    expected, found
                ),
                sutra_ref: Some("1.1.2".to_string()),
                hint: Some("Sahi Vibhakti pratyaya lagao.".to_string()),
            },
            TypeCheckError::SatyaasatyaApekshita(msg) => Diagnostic {
                severity: Severity::Dosha,
                code: "D003".to_string(),
                sanskrit_title: "सत्यासत्य-अपेक्षित".to_string(),
                roman_title: "Satyaasatya Apeksita".to_string(),
                message: format!("Satyasatya (Bool) अपेक्षित है: {}", msg),
                sutra_ref: Some("1.1.3".to_string()),
                hint: Some("Yadi/Yavat ki sthiti Satyasatya honi chahiye.".to_string()),
            },
            TypeCheckError::PrakaaraAsangata(msg) => Diagnostic {
                severity: Severity::Dosha,
                code: "D011".to_string(),
                sanskrit_title: "असंगत प्रकार".to_string(),
                roman_title: "Prakara Asangata".to_string(),
                message: format!("Yaha prakara asangata hai: {}", msg),
                sutra_ref: None,
                hint: None,
            },
            TypeCheckError::AnavasthaDosha { dhatu_name } => Diagnostic {
                severity: Severity::Dosha,
                code: "D040".to_string(),
                sanskrit_title: "अनवस्था-दोषः".to_string(),
                roman_title: "Anavastha Doshah".to_string(),
                message: format!(
                    "'{}' — is Dhātu mein koi base case nahi mila jo recursion ko rokta ho. \
                     Har path recursive call tak jaata hai — infinite regress ka khatra hai.",
                    dhatu_name
                ),
                sutra_ref: Some(
                    "Nyaya Sutra + Ashtadhyayi 6.4.22 (finality of rule application)".to_string(),
                ),
                hint: Some(format!(
                    "'{}' ke andar ek Yadi/Anyatha branch add karo jisme kam se kam ek path \
                     bina recursive call ke terminate ho.",
                    dhatu_name
                )),
            },
            TypeCheckError::PanktiAsangata { expected, found } => Diagnostic {
                severity: Severity::Dosha,
                code: "D050".to_string(),
                sanskrit_title: "पङ्क्ति-असङ्गति".to_string(),
                roman_title: "Pankti Asangati".to_string(),
                message: format!(
                    "Pankti mein element types saman hone chahiye. \
                     Expected {:?}, found {:?}.",
                    expected, found
                ),
                sutra_ref: Some("Ashtadhyayi 1.2.64 (samanya-vishesha)".to_string()),
                hint: None,
            },
            TypeCheckError::VinyasaAprayukta { found } => Diagnostic {
                severity: Severity::Dosha,
                code: "D051".to_string(),
                sanskrit_title: "विन्यास-अप्रयुक्तः".to_string(),
                roman_title: "Vinyasa Aprayuktah".to_string(),
                message: format!(
                    "Indexing operation sirf array (Pankti) par lagu hota hai. \
                     Found non-array type {:?}.",
                    found
                ),
                sutra_ref: None,
                hint: None,
            },
            TypeCheckError::VinyasaSimaLanghana { index, len } => Diagnostic {
                severity: Severity::Dosha,
                code: "D052".to_string(),
                sanskrit_title: "विन्यास-सीमा-लङ्घनम्".to_string(),
                roman_title: "Vinyasa Sima Langhanam".to_string(),
                message: format!(
                    "Array index {} array ki length {} se adhik hai. \
                     Valid indices 0 se {} tak hain.",
                    index,
                    len,
                    len - 1
                ),
                sutra_ref: None,
                hint: None,
            },
TypeCheckError::KramashahAprayukta { found } => Diagnostic {
                severity: Severity::Dosha,
                code: "D053".to_string(),
                sanskrit_title: "क्रमशः-अप्रयुक्तः".to_string(),
                roman_title: "Kramasah Aprayuktah".to_string(),
                message: format!(
                    "kramasah requires a Pankti (array) as the iterable; found {:?}",
                    found
                ),
                sutra_ref: Some("Kramapatha (Vedic pairwise sequential recitation)".to_string()),
                hint: None,
            },
            TypeCheckError::AvaliAsangata { expected, found } => Diagnostic {
                severity: Severity::Dosha,
                code: "D054".to_string(),
                sanskrit_title: "आवलि-असङ्गति".to_string(),
                roman_title: "Avali Asangati".to_string(),
                message: format!(
                    "Avali mein element types saman hone chahiye. \
                     Expected {:?}, found {:?}.",
                    expected, found
                ),
                sutra_ref: Some("Ashtadhyayi 1.2.64 (samanya-vishesha)".to_string()),
                hint: None,
            },
            TypeCheckError::PrakshepaAprayukta { found } => Diagnostic {
                severity: Severity::Dosha,
                code: "D055".to_string(),
                sanskrit_title: "प्रक्षेप-अप्रयुक्तः".to_string(),
                roman_title: "Prakshepa Aprayuktah".to_string(),
                message: format!(
                    "Prakshepa-dhatu (push) operation Avali type ke saath hi prayukt hota hai. \
                     Found non-Avali type {:?}.",
                    found
                ),
                sutra_ref: None,
                hint: None,
            },
            TypeCheckError::ApakarshanaAprayukta { found } => Diagnostic {
                severity: Severity::Dosha,
                code: "D056".to_string(),
                sanskrit_title: "अपकर्षण-अप्रयुक्तः".to_string(),
                roman_title: "Apakarshana Aprayuktah".to_string(),
                message: format!(
                    "Apakarshana-dhatu (pop) operation Avali type ke saath hi prayukt hota hai. \
                     Found non-Avali type {:?}.",
                    found
                ),
                sutra_ref: None,
                hint: None,
            },
            TypeCheckError::DravyaApariyata { name } => Diagnostic {
                severity: Severity::Dosha,
                code: "D057".to_string(),
                sanskrit_title: "द्रव्य-अपरिज्ञातम्".to_string(),
                roman_title: "Dravya Aparijnatam".to_string(),
                message: format!(
                    "Dravya (struct) type '{}' parichit nahi hai. \
                     Pehle ise define karo ya spelling check karo.",
                    name
                ),
                sutra_ref: Some(
                    "Vaiśeṣika Sūtra (Dravya as one of the seven categories)".to_string()
                ),
                hint: Some(format!("'{}' naam ka dravya pehle define karo.", name)),
            },
            TypeCheckError::AngaApraapya { dravya_name, anga_name } => Diagnostic {
                severity: Severity::Dosha,
                code: "D058".to_string(),
                sanskrit_title: "अङ्ग-अप्राप्यम्".to_string(),
                roman_title: "Anga Aprapyam".to_string(),
                message: format!(
                    "Struct '{}' mein '{}' naam ka anga (field) nahi hai.",
                    dravya_name, anga_name
                ),
                sutra_ref: Some(
                    "Vaiśeṣika Sūtra (Anga as constituent of Dravya)".to_string()
                ),
                hint: Some(format!("'{}' ke defined angas check karo.", dravya_name)),
            },
            TypeCheckError::SamavayaAprayukta { found } => Diagnostic {
                severity: Severity::Dosha,
                code: "D059".to_string(),
                sanskrit_title: "समवाय-अप्रयुक्तः".to_string(),
                roman_title: "Samavaya Aprayuktah".to_string(),
                message: format!(
                    "Samavaya (field access) sirf Dravya (struct) par apply hota hai. \
                      Found type: {}.",
                    found
                ),
                sutra_ref: Some(
                    "Vaiśeṣika Sūtra (Samavaya as inherence relation)".to_string()
                ),
                hint: None,
            },
            TypeCheckError::NirmanaAsangati { dravya_name, expected_count, found_count, anga_name, position, expected_type, found_type } => {
                let message = if expected_count != found_count {
                    format!(
                        "Nirmāṇa (struct instantiation) mein values ki sankhya match nahi \
                         karti. Struct '{}' ke liye {} values apekshit the, par {} mile.",
                        dravya_name, expected_count, found_count
                    )
                } else {
                    format!(
                        "Nirmāṇa (struct instantiation) mein field '{}' (sthaan {}) ki \
                         prakara asangata hai struct '{}' mein. Expected {:?}, found {:?}.",
                        anga_name, position, dravya_name, expected_type, found_type
                    )
                };
                Diagnostic {
                    severity: Severity::Dosha,
                    code: "D060".to_string(),
                    sanskrit_title: "निर्माण-असङ्गति".to_string(),
                    roman_title: "Nirmana Asangati".to_string(),
                    message,
                    sutra_ref: None,
                    hint: None,
                }
            }
        }
    }

    pub fn from_codegen_error(err: &CodegenError) -> Diagnostic {
        match err {
            CodegenError::UnsupportedNode(n) => Diagnostic {
                severity: Severity::Dosha,
                code: "D004".to_string(),
                sanskrit_title: "असमर्थित पद".to_string(),
                roman_title: "Asamarthita Pada".to_string(),
                message: format!(
                    "'{}' — yeh pada abhi codegen mein \
                                  samarthit nahi.",
                    n
                ),
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
                message: format!(
                    "'{}' file nahi mili ya padhi nahi ja \
                                  sakti.",
                    msg
                ),
                sutra_ref: None,
                hint: Some(
                    "Sahi file path do: devvani compile \
                            <file.dvn>"
                        .to_string(),
                ),
            },
            CompilerError::LexError(msg) => Diagnostic {
                severity: Severity::Dosha,
                code: "D008".to_string(),
                sanskrit_title: "वर्ण-विश्लेषण-दोष".to_string(),
                roman_title: "Varna Vishleshan Dosha".to_string(),
                message: format!("Shabda pahchana mein samasya: {}", msg),
                sutra_ref: Some("1.1.1".to_string()),
                hint: Some(
                    "IAST Unicode sahi hai? \
                            Matra aur anusvara check karo."
                        .to_string(),
                ),
            },
            CompilerError::ParseError(msg) => Diagnostic {
                severity: Severity::Dosha,
                code: "D009".to_string(),
                sanskrit_title: "वाक्य-संरचना-दोष".to_string(),
                roman_title: "Vakya Sanrachna Dosha".to_string(),
                message: format!("SOV krama galat hai: {}", msg),
                sutra_ref: Some("2.1.1".to_string()),
                hint: Some(
                    "Devvani SOV order follow karo: \
                            Kartā Karma Kriyā."
                        .to_string(),
                ),
            },
            CompilerError::CodegenError(msg) => Diagnostic {
                severity: Severity::Dosha,
                code: "D010".to_string(),
                sanskrit_title: "कोड-निर्माण-दोष".to_string(),
                roman_title: "Code Nirman Dosha".to_string(),
                message: format!(
                    "Rust code generation mein samasya: \
                                  {}",
                    msg
                ),
                sutra_ref: None,
                hint: None,
            },
        }
    }

    pub fn report(diagnostics: &[Diagnostic]) -> String {
        if diagnostics.is_empty() {
            return "✓ Shuddham — कोई दोष नही | No errors found.\n".to_string();
        }
        diagnostics
            .iter()
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
    fn test_from_type_error_naama_apraapta() {
        let err = TypeCheckError::NaamaApraapta("ramah".to_string());
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
        let err1 = TypeCheckError::NaamaApraapta("ramah".to_string());
        let diag1 = DiagnosticEngine::from_type_error(&err1);
        let err2 = CompilerError::ParseError("test".to_string());
        let diag2 = DiagnosticEngine::from_compiler_error(&err2);

        let report = DiagnosticEngine::report(&[diag1, diag2]);
        assert!(report.contains("D001"));
        assert!(report.contains("D009"));
    }

    #[test]
    fn test_from_type_error_anavastha_dosha() {
        let err = TypeCheckError::AnavasthaDosha {
            dhatu_name: "recur".to_string(),
        };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D040");
        assert!(diag.message.contains("recur"));
        assert!(diag.display().contains("recur"));
        assert!(diag.hint.unwrap().contains("recur"));
    }

    #[test]
    fn test_from_type_error_pankti_asangata() {
        let err = TypeCheckError::PanktiAsangata {
            expected: devvani_typesystem::DevvaniType::Subject("Purnaank".to_string()),
            found: devvani_typesystem::DevvaniType::Vaak,
        };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D050");
        assert!(diag.display().contains("Pankti Asangati"));
        assert!(diag.sutra_ref.unwrap().contains("Ashtadhyayi 1.2.64"));
    }

    #[test]
    fn test_from_type_error_vinyasa_aprayukta() {
        let err = TypeCheckError::VinyasaAprayukta {
            found: devvani_typesystem::DevvaniType::Subject("Purnaank".to_string()),
        };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D051");
        assert!(diag.display().contains("Vinyasa Aprayuktah"));
    }

    #[test]
    fn test_from_type_error_vinyasa_sima_langhana() {
        let err = TypeCheckError::VinyasaSimaLanghana { index: 5, len: 3 };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D052");
        assert!(diag.display().contains("Vinyasa Sima Langhanam"));
        assert!(diag.message.contains("5"));
        assert!(diag.message.contains("3"));
    }

    #[test]
    fn test_from_type_error_kramasah_aprayukta() {
        let err = TypeCheckError::KramashahAprayukta {
            found: devvani_typesystem::DevvaniType::Subject("Purnaank".to_string()),
        };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D053");
        assert!(diag.display().contains("Kramasah Aprayuktah"));
        assert!(diag.sanskrit_title.contains("क्रमशः-अप्रयुक्तः"));
    }

    #[test]
    fn test_from_type_error_kramasah_diagnostics_d053() {
        let err = TypeCheckError::KramashahAprayukta {
            found: devvani_typesystem::DevvaniType::Subject("Purnaank".to_string()),
        };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D053");
        assert!(diag.message.contains("Pankti (array) as the iterable"));
        assert!(diag.sutra_ref.unwrap().contains("Kramapatha"));
    }

    #[test]
    fn test_from_type_error_avali_asangata() {
        let err = TypeCheckError::AvaliAsangata {
            expected: devvani_typesystem::DevvaniType::Subject("Purnaank".to_string()),
            found: devvani_typesystem::DevvaniType::Vaak,
        };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D054");
        assert!(diag.display().contains("Avali Asangati"));
        assert!(diag.sutra_ref.unwrap().contains("Ashtadhyayi 1.2.64"));
    }

    #[test]
    fn test_from_type_error_prakshepa_aprayukta() {
        let err = TypeCheckError::PrakshepaAprayukta {
            found: devvani_typesystem::DevvaniType::Subject("Purnaank".to_string()),
        };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D055");
        assert!(diag.display().contains("Prakshepa Aprayuktah"));
        assert!(diag.sanskrit_title.contains("प्रक्षेप-अप्रयुक्तः"));
    }

    #[test]
    fn test_from_type_error_apakarshana_aprayukta() {
        let err = TypeCheckError::ApakarshanaAprayukta {
            found: devvani_typesystem::DevvaniType::Subject("Purnaank".to_string()),
        };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D056");
        assert!(diag.display().contains("Apakarshana Aprayuktah"));
        assert!(diag.sanskrit_title.contains("अपकर्षण-अप्रयुक्तः"));
    }

    #[test]
    fn test_from_type_error_dravya_apariyata() {
        let err = TypeCheckError::DravyaApariyata {
            name: "gadha".to_string(),
        };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D057");
        assert!(diag.display().contains("Dravya Aparijnatam"));
        assert!(diag.sanskrit_title.contains("द्रव्य-अपरिज्ञातम्"));
        assert!(diag.sutra_ref.unwrap().contains("Vaiśeṣika Sūtra"));
    }

    #[test]
    fn test_from_type_error_anga_apraapya() {
        let err = TypeCheckError::AngaApraapya {
            dravya_name: "manushya".to_string(),
            anga_name: "agaj".to_string(),
        };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D058");
        assert!(diag.display().contains("Anga Aprapyam"));
        assert!(diag.sanskrit_title.contains("अङ्ग-अप्राप्यम्"));
        assert!(diag.sutra_ref.unwrap().contains("Vaiśeṣika Sūtra"));
    }

    #[test]
    fn test_from_type_error_samavaya_aprayukta() {
        let err = TypeCheckError::SamavayaAprayukta {
            found: "Purnaank".to_string(),
        };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D059");
        assert!(diag.display().contains("Samavaya Aprayuktah"));
        assert!(diag.sanskrit_title.contains("समवाय-अप्रयुक्तः"));
        assert!(diag.sutra_ref.unwrap().contains("Vaiśeṣika Sūtra"));
    }

    #[test]
    fn test_from_type_error_nirmana_asangati_count() {
        let err = TypeCheckError::NirmanaAsangati {
            dravya_name: "manushya".to_string(),
            expected_count: 2,
            found_count: 1,
            anga_name: String::new(),
            position: 0,
            expected_type: devvani_typesystem::DevvaniType::Unknown,
            found_type: devvani_typesystem::DevvaniType::Unknown,
        };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D060");
        assert!(diag.display().contains("Nirmana Asangati"));
        assert!(diag.sanskrit_title.contains("निर्माण-असङ्गति"));
        assert!(diag.message.contains("2 values"));
        assert!(diag.message.contains("1 mile"));
        assert!(diag.message.contains("manushya"));
    }

    #[test]
    fn test_from_type_error_nirmana_asangati_type() {
        let err = TypeCheckError::NirmanaAsangati {
            dravya_name: "manushya".to_string(),
            expected_count: 2,
            found_count: 2,
            anga_name: "sankhya".to_string(),
            position: 1,
            expected_type: devvani_typesystem::DevvaniType::Subject("Purnaank".to_string()),
            found_type: devvani_typesystem::DevvaniType::Vaak,
        };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D060");
        assert!(diag.display().contains("Nirmana Asangati"));
        assert!(diag.message.contains("sthaan 1"));
        assert!(diag.message.contains("sankhya"));
        assert!(diag.message.contains("manushya"));
    }
}
