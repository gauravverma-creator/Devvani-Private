use crate::CompilerError;
use devvani_codegen::CodegenError;
use devvani_parser::ParseError;
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
            TypeCheckError::PhalaVisamgati { expected, found } => Diagnostic {
                severity: Severity::Dosha,
                code: "D061".to_string(),
                sanskrit_title: "फलविसंगति".to_string(),
                roman_title: "Phala Visamgati".to_string(),
                message: format!(
                    "Arogya/Dosha mein praapt prakaar anukool nahi hai. \
                     Phalam success/error type se match nahi karta: \
                     expected {:?}, found {:?}.",
                    expected, found
                ),
                sutra_ref: Some(
                    "Charaka Samhita, Nidana Pancaka (five-fold examination)".to_string()
                ),
                hint: Some(
                    "Nidana ke phalam prakaar se arogya/dosha ki prakara saman rakho.".to_string()
                ),
            },
            TypeCheckError::NidanaAparichaya => Diagnostic {
                severity: Severity::Dosha,
                code: "D062".to_string(),
                sanskrit_title: "निदानअपरिचय".to_string(),
                roman_title: "Nidana Aparichaya".to_string(),
                message: "Nidana ka lakshya Phalam type nahi hai. Nidana ko Phalam par aropit karna chahiye.".to_string(),
                sutra_ref: Some(
                    "Charaka Samhita, Nidana Pancaka (Nidana as knowing the disease)".to_string()
                ),
                hint: Some(
                    "Nidana ke samne Phalam type ka expression rakho.".to_string()
                ),
            },
            TypeCheckError::PancakaAvishishtata => Diagnostic {
                severity: Severity::Dosha,
                code: "D063".to_string(),
                sanskrit_title: "पञ्चकअविशिष्टता".to_string(),
                roman_title: "Pancaka Avishishtata".to_string(),
                message: "Nidana ke donon bhujae (arogya aur dosha) upasthit honi chahiye. \
                          Abhi ek ya donon missing hain.".to_string(),
                sutra_ref: Some(
                    "Charaka Samhita, Nidana Pancaka (five-fold examination completeness)".to_string()
                ),
                hint: Some(
                    "Nidana mein arogya-bind aur dosha-bind donon provide karo.".to_string()
                ),
            },
            TypeCheckError::SamprāptiAyogyatā => Diagnostic {
                severity: Severity::Dosha,
                code: "D064".to_string(),
                sanskrit_title: "सम्प्राप्तिअयोग्यता".to_string(),
                roman_title: "Samprāpti Ayogyatā".to_string(),
                message: "Samprapti (?) operation sirf Phalam type return karne wale \
                          Dhātu ke andar hi prayukt hota hai.".to_string(),
                sutra_ref: Some(
                    "Charaka Samhita, Nidana Pancaka (Samprapti as path of disease)".to_string()
                ),
                hint: Some(
                    "Is Dhātu ka return_type Phalam banāo ya Samprapti hatao.".to_string()
                ),
            },
            TypeCheckError::DoshaAsangati { expected, found } => Diagnostic {
                severity: Severity::Dosha,
                code: "D065".to_string(),
                sanskrit_title: "दोषअसङ्गति".to_string(),
                roman_title: "Dosha Asangati".to_string(),
                message: format!(
                    "Samprapti propagate kar raha hai error type {:?}, \
                     par enclosing Dhātu mein {:?} apekshit hai. \
                     Error types asangat hain.",
                    found, expected
                ),
                sutra_ref: Some(
                    "Charaka Samhita, Nidana Pancaka (Dosha incompatibility)".to_string()
                ),
                hint: Some(
                    "Samprapti ke target Phalam ka error type enclosing Dhātu ke \
                     return error type se mel khawe.".to_string()
                ),
            },
            TypeCheckError::PhalaSandarbhaAbhava => Diagnostic {
                severity: Severity::Dosha,
                code: "D066".to_string(),
                sanskrit_title: "फलसन्दर्भअभाव".to_string(),
                roman_title: "Phala Sandarbha Abhava".to_string(),
                message: "Arogya ya Dosha upyog karte samay Phalam ka sandarbh \
                          nahi mila. Yeh Nidana ke andar ya Phalam-returning \
                          Dhātu ke shesh mein hi upyukt hai.".to_string(),
                sutra_ref: Some(
                    "Charaka Samhita, Nidana Pancaka (absence of diagnostic context)".to_string()
                ),
                hint: Some(
                    "Arogya/Dosha ko Nidana ke andar ya Phalam return karne wale \
                     Dhātu ke ant mein use karo.".to_string()
                ),
            },
            TypeCheckError::SvatvaBhanga { name } => Diagnostic {
                severity: Severity::Dosha,
                code: "D067".to_string(),
                sanskrit_title: "स्वत्वभङ्ग".to_string(),
                roman_title: "Svatva Bhanga".to_string(),
                message: format!(
                    "'{}' ka Svatva (ownership) already move ho chuka hai — \
                     ise dobara upyog nahi kiya ja sakta. Mimāṃsā vyākhyāna: \
                     Svatva Dharmasāstra ke anusaar ek varth vartamāna ek hi \
                     śāstā ke adhikār mein hota hai.",
                    name
                ),
                sutra_ref: Some(
                    "Mīmāṃsā Sūtra 2.1.14 (Svatva — ownership jurisprudence)".to_string()
                ),
                hint: Some(format!(
                    "'{}' ko pehle define karo ya uska sandarbha (borrow) \
                     istemal karo.",
                    name
                )),
            },
            TypeCheckError::AdhikaraDvandva { name } => Diagnostic {
                severity: Severity::Dosha,
                code: "D068".to_string(),
                sanskrit_title: "अधिकारद्वन्द्व".to_string(),
                roman_title: "Adhikara Dvandva".to_string(),
                message: format!(
                    "'{}' par ek active borrow ke saath ek aur borrow\n        \
                     laga sakte ho nahi — ya toh do bojhik borrows\n        \
                     (do immutable) hain toh ek hi hona chahiye, ya ek\n        \
                     mutable borrow aktiv hai toh koi aur borrow nahi\n        \
                     ho sakta. Pāṇinian vyākhyāna: Adhikār eligibility\n        \
                     hai aur do adhikār ek saath clash karte hain.",
                    name
                ),
                sutra_ref: Some(
                    "Ashtadhyayi 2.1.1 (Adhikara — eligibility-rule term)".to_string()
                ),
                hint: Some(format!(
                    "'{}' ka pehla borrow close karke dobara try karo.",
                    name
                )),
            },
            TypeCheckError::KshayaAnantaraUpayoga { name } => Diagnostic {
                severity: Severity::Dosha,
                code: "D069".to_string(),
                sanskrit_title: "क्षयानन्तरउपयोग".to_string(),
                roman_title: "Kshaya Anantara Upayoga".to_string(),
                message: format!(
                    "'{}' ko use kiya ja raha hai jab takki iska Kshaya\n        \
                     (scope exit) ho chuka hai. Pāṇinian vyākhyāna: \n        \
                     Kshaya = scope exit, upayoga = use — scope ke bahar\n        \
                     jo naam available nahi hai uska upyog doosara dosh hai.",
                    name
                ),
                sutra_ref: Some(
                    "Mīmāṃsā + Pāṇini (Kshaya scope-rule term)".to_string()
                ),
                hint: Some(format!(
                    "'{}' ko is scope ke andar hi use karo ya scope ke\n        \
                     bahar pehle declare karo.",
                    name
                )),
            },
             TypeCheckError::VikaraAdhikaraDvaya { name } => Diagnostic {
                severity: Severity::Dosha,
                code: "D070".to_string(),
                sanskrit_title: "विकारअधिकारद्वय".to_string(),
                roman_title: "Vikara Adhikara Dvaya".to_string(),
                message: format!(
                    "'{}' par do simultaneous mutable borrows (vikara adhikara\n        \
                     dvaya) exist kar rahe hain. Mimāṃsā vyākhyāna: एकस्य \n        \
                     स्वत्वम् (ekasya svaratvam) — ek varth ki svatva ek baar hi \n        \
                     upyog hogi, do mutable borrows karo ki nahi.",
                    name
                ),
                sutra_ref: Some(
                    "Mīmāṃsā Sūtra + Ashtadhyayi (single mutable borrow rule)".to_string()
                ),
                hint: Some(format!(
                    "'{}' ka pehla mutable borrow close karke dobara try karo.",
                    name
                )),
            },
            TypeCheckError::SamanyaAnishchitaDvandva { name, param_name, found_type, previous_type } => Diagnostic {
                severity: Severity::Dosha,
                code: "D071".to_string(),
                sanskrit_title: "सामान्यअनिश्चितद्वन्द्व".to_string(),
                roman_title: "Samanya Anishchita Dvandva".to_string(),
                message: format!(
                    "sāmānya param '{}' par '{}' mein conflicting inference ho rahi hai: \
                     ek jagah {:?} aur doosri jagah {:?} mil rahe hain. \
                     Vaiśeṣika vyākhyāna: sāmānya (generic) ki ek vishista pratirupa \
                     ek hi ho sakta hai.",
                    param_name, name, found_type, previous_type
                ),
                sutra_ref: Some(
                    "Vaiśeṣika Sūtra (sāmānya-viśeṣa type-resolution)".to_string()
                ),
                hint: None,
            },
            TypeCheckError::SamanyaAniyata { name, param_name } => Diagnostic {
                severity: Severity::Dosha,
                code: "D072".to_string(),
                sanskrit_title: "सामान्यअनियता".to_string(),
                roman_title: "Samanya Aniyata".to_string(),
                message: format!(
                    "sāmānya param '{}' '{}' ke phalam-type mein upyog kiya ja raha hai, \
                     lekin usse call ke argument se anumey nahi kiya ja sakta. \
                     Vaiśeṣika vyākhyāna: sāmānya ka visheṣa (call arguments) se \
                     niścaya (niscaya) avasyaka hai.",
                    param_name, name
                ),
                sutra_ref: Some(
                    "Vaiśeṣika Sūtra (inferability-at-call-site doctrine)".to_string()
                ),
                hint: None,
            },
            TypeCheckError::AnumanaViphalata => Diagnostic {
                severity: Severity::Dosha,
                code: "D073".to_string(),
                sanskrit_title: "अनुमानविफलता".to_string(),
                roman_title: "Anumana Viphalata".to_string(),
                message: "Anumāṇa (type inference) is kshīṇa — expression se prakara \
                          nirdhāraṇa nahi ho sakā. Nyāya vyākhyāna: anumāna ki \
                          viphalatā (failure of inference) tab hotī hai jābartha \
                          hetu (premise) aprakaṭa ho."
                    .to_string(),
                sutra_ref: Some(
                    "Nyāya Sūtra (Anumāna pramāṇa — inference failure doctrine)".to_string()
                ),
                hint: Some(
                    "Is expression ko ek spaṣṭa type ke saath declare karo ya \
                     uske mūla (operands) ke types suniścit karo."
                        .to_string(),
                ),
            },
            TypeCheckError::AnumanaSamgatiBhanga => Diagnostic {
                severity: Severity::Dosha,
                code: "D074".to_string(),
                sanskrit_title: "अनुमानसंगतिभङ्ग".to_string(),
                roman_title: "Anumana Samgati Bhanga".to_string(),
                message: "Anumāṇa-ke anek mārga (return paths) mein prakara \
                          asaman hai — ek mārga se eka prakaara, doosre mārga \
                          se anya prakaara praapt hai. Nyāya vyākhyāna: \
                          anumāṇa-saṃgati-bhaṅga (fragmentation of inference) \
                          tab hotī hai jābartha vibhinna pathon se \
                          viruddha anumeya nishchit hota hai."
                    .to_string(),
                sutra_ref: Some(
                    "Nyāya Sūtra (Anumāna pramāṇa — conflicting inference across paths)".to_string()
                ),
                hint: Some(
                    "Sab return paths ko eka samaan type ke banāo ya function ke \
                      return type ko explicitly declare karo."
                        .to_string(),
                ),
            },
            TypeCheckError::PraptiAprayukta { found } => Diagnostic {
                severity: Severity::Dosha,
                code: "D075".to_string(),
                sanskrit_title: "प्राप्त्यप्रयुक्त".to_string(),
                roman_title: "Prapti Aprayukta".to_string(),
                message: format!(
                    "Prapti (join) sirf Samyoga (thread handle) type par prayukt hota hai. \
                     Found type: {:?}. Nyaya-Vaisheshika: Samyoga = conjunction/attainment \
                     of a result through connection.",
                    found
                ),
                sutra_ref: Some(
                    "Nyaya-Vaisheshika (Samyoga — conjunction/attainment doctrine)".to_string()
                ),
                hint: Some(
                    "Prapti ko Samyoga type ke handle ke saath hi upyog karo.".to_string()
                ),
            },
            TypeCheckError::DutaBhejAprayukta { found } => {
                Diagnostic {
                    severity: Severity::Dosha,
                    code: "D076".to_string(),
                    sanskrit_title: "दूतभेजअप्रयुक्त".to_string(),
                    roman_title: "Duta Bhej Aprayukta".to_string(),
                    message: format!(
                        "Bhej (send) operation sirf DutaBhejaka (channel sender) type par \
                         prayukt hota hai. Found type: {:?}. Sandesha-kavya tradition: \
                         Duta (messenger) carries the message from sender to receiver.",
                        found
                    ),
                    sutra_ref: Some(
                        "Sandesha-kavya / Nyaya-Vaisheshika (messenger-conjunction context)".to_string()
                    ),
                    hint: Some(
                        "Bhej ko DutaBhejaka type ke variable ke saath hi call karo.".to_string()
                    ),
                }
            }
            TypeCheckError::DutaGrahanAprayukta { found } => Diagnostic {
                severity: Severity::Dosha,
                code: "D077".to_string(),
                sanskrit_title: "दूतग्रहणअप्रयुक्त".to_string(),
                roman_title: "Duta Grahan Aprayukta".to_string(),
                message: format!(
                    "Grahan karo (receive) operation sirf DutaGrahaka (channel receiver) \
                     type par prayukt hota hai. Found type: {:?}. Sandesha-kavya tradition: \
                     Grahaka is the recipient who receives the messenger's message.",
                    found
                ),
                sutra_ref: Some(
                    "Sandesha-kavya / Nyaya-Vaisheshika (recipient-conjunction context)".to_string()
                ),
                hint: Some(
                    "Grahan karo ko DutaGrahaka type ke variable ke saath hi call karo.".to_string()
                ),
            },
            TypeCheckError::ManasAprayukta { found } => Diagnostic {
                severity: Severity::Dosha,
                code: "D078".to_string(),
                sanskrit_title: "मनसअप्रयुक्त".to_string(),
                roman_title: "Manas Aprayukta".to_string(),
                message: format!(
                    "Manas (mutex-guarded block) sirf Manas (mutex-guarded) type par \
                     prayukt hota hai. Found type: {:?}. Nyaya Sutra: Manas is the \
                     internal instrument that sequentially coordinates cognitions \
                     (NS 1.1.16 — manah parikshah).",
                    found
                ),
                sutra_ref: Some(
                    "Nyaya Sutra NS 1.1.16 (manas — sequential-cognition doctrine)".to_string()
                ),
                hint: Some(
                    "Manas ko Manas type ke mutex variable ke saath hi upyog karo.".to_string()
                ),
            },
            TypeCheckError::DharaVinyasaAsangati { found } => Diagnostic {
                severity: Severity::Dosha,
                code: "D079".to_string(),
                sanskrit_title: "धाराविन्यासासंगति".to_string(),
                roman_title: "Dhara Vinyasa Asangati".to_string(),
                message: format!(
                    "Dhara (dhāraṇa / binding) mein vinyāsa (pattern) asangata hai: \
                     bahu-naama (multi-name) destructuring ke liye Duta (sender, receiver) \
                     pair type chahiye. Found type: {:?}. Nyaya Sutra: vinyāsa śakti \
                     binds multiple artha-s (meanings) only to a dvandva (pair) \
                     (NS 2.2.33 — dvandva samāsa doctrine).",
                    found
                ),
                sutra_ref: Some(
                    "Nyaya Sutra NS 2.2.33 (dvandva samasa — compound-of-two doctrine)".to_string()
                ),
                hint: Some(
                    "Bahu-naama dhara binding ke liye `duta banaa` ya koi Duta pair type \
                     expression hi upyog karo.".to_string()
                ),
            },
            TypeCheckError::ParinamaAsangati { stage, expected, found } => Diagnostic {
                severity: Severity::Dosha,
                code: "D080".to_string(),
                sanskrit_title: "परिणामासंगति".to_string(),
                roman_title: "Parinama Asangati".to_string(),
                message: format!(
                    "Pariṇāma chain stage {} mein prakara asangata hai: \
                     expected {:?}, found {:?}. Sāṃkhya pariṇāma-vāda: \
                     each transformation must accept the prior stage's output type.",
                    stage, expected, found
                ),
                sutra_ref: Some(
                    "Sāṃkhya (pariṇāma-vāda — sequential transformation doctrine)".to_string()
                ),
                hint: Some(
                    "Pariṇāma chain ke har stage ka input type pehle ke stage ke \
                     output type se mel khawe.".to_string()
                ),
            },
            TypeCheckError::ParinamaShunya => Diagnostic {
                severity: Severity::Dosha,
                code: "D081".to_string(),
                sanskrit_title: "परिणामशून्यता".to_string(),
                roman_title: "Parinama Shunyata".to_string(),
                message: "`pariṇāma []` — shunya dhatu-chain ka upayog \
                          kar rahe ho jahan concrete type apekshit hai. \
                          Pariṇāma-vāda: shunya transformations se koi phalam nahi.".to_string(),
                sutra_ref: Some(
                    "Sāṃkhya (pariṇāma-vāda — non-empty transformation sequence)".to_string()
                ),
                hint: Some(
                    "Pariṇāma mein kam se kam ek dhatu likho: `x pariṇāma [f]`.".to_string()
                ),
            },
             TypeCheckError::ParinamaDoshaVaisamya { error_a, error_b } => Diagnostic {
                 severity: Severity::Dosha,
                 code: "D082".to_string(),
                 sanskrit_title: "परिणामदोषवैषम्य".to_string(),
                 roman_title: "Parinama Dosha Vaisamya".to_string(),
                 message: format!(
                     "Pariṇāma chain mein do ya adhik fallible dhatus ke \
                      error types asangat hain: {:?} vs {:?}. \
                      Devvani koi automatic coercion nahi karta.",
                     error_a, error_b
                 ),
                 sutra_ref: Some(
                     "Sāṃkhya (pariṇāma-vāda — uniform error-type propagation)".to_string()
                 ),
                 hint: Some(
                     "Chain ke sab fallible dhatus ko ek saman error type \
                      ke saath define karo ya Phalam types align karo.".to_string()
                 ),
             },
             TypeCheckError::NigamanaNotBool { found } => Diagnostic {
                 severity: Severity::Dosha,
                 code: "D086".to_string(),
                 sanskrit_title: "निगमनाभिन्नप्रकार".to_string(),
                 roman_title: "Nigamana Abhinna Prakaar".to_string(),
                 message: format!(
                     "Nigamana (assert-true) ka argument Satyasatya (Bool) \
                      hona chahiye, par {:?} praapt hua. Nyāya vyākhyāna: \
                      nigamana ka siddhānta kevala satya ya asatya ho sakta hai.",
                     found
                 ),
                 sutra_ref: Some(
                     "Nyāya Sūtra (Nigamana — five-membered syllogism conclusion)".to_string()
                 ),
                 hint: Some(
                     "Nigamana ka argument Bool expression do.".to_string()
                 ),
             },
             TypeCheckError::SadrishyaNigamanaMismatchedTypes { left, right } => Diagnostic {
                 severity: Severity::Dosha,
                 code: "D087".to_string(),
                 sanskrit_title: "सादृश्यनिगमनवैषम्य".to_string(),
                 roman_title: "Sadrishya Nigamana Vaisamya".to_string(),
                 message: format!(
                     "Sadrishya-nigamana (assert-equal) ke donon arguments \
                      ki prakara saman honi chahiye. Left: {:?}, right: {:?}. \
                      Nyāya vyākhyāna: sadrishya (similarity) ke liye donon \
                      vastuen saman prakaar ki honi chahiye.",
                     left, right
                 ),
                 sutra_ref: Some(
                     "Nyāya Sūtra (Sadrishya — similarity doctrine)".to_string()
                 ),
                 hint: Some(
                     "Dono arguments ko ek saman type ke banayo.".to_string()
                 ),
             },
             TypeCheckError::SadrishyaNigamanaNotEqualityComparable { ty } => Diagnostic {
                 severity: Severity::Dosha,
                 code: "D088".to_string(),
                 sanskrit_title: "सादृश्यनिगमनासमर्थ".to_string(),
                 roman_title: "Sadrishya Nigamana Asamartha".to_string(),
                 message: format!(
                     "Sadrishya-nigamana (assert-equal) ke argument ka \
                      type {:?} tulya (equality) comparison ke liye samartha \
                      nahi hai. Nyāya vyākhyāna: samanya jati mein tulya \
                      hetu upayog nahi hota.",
                     ty
                 ),
                 sutra_ref: Some(
                     "Nyāya Sūtra (Tulya — equality comparison doctrine)".to_string()
                 ),
                 hint: Some(
                     "Eq/PartialEq implement karne wale type ka upayog karo.".to_string()
                 ),
             },
              TypeCheckError::ParikshaaBodyNotUnit => Diagnostic {
                  severity: Severity::Dosha,
                  code: "D089".to_string(),
                  sanskrit_title: "परीक्षाशरीरवैषम्य".to_string(),
                  roman_title: "Parikshaa Sharira Vaisamya".to_string(),
                  message: "Parikshaa (test) ka shareera (body) shunya-prakaar \
                            (unit/void) return karna chahiye, par koi mulya \
                            (value) produce kar raha hai. Nyāya vyākhyāna: \
                            parikshaa ka phalam kevala siddha (established) ya \
                            asiddha (unestablished) hota hai, na ki koi dravya.".to_string(),
                  sutra_ref: Some(
                      "Nyāya Sūtra (Parikshaa — examination doctrine)".to_string()
                  ),
                  hint: Some(
                      "Parikshaa body ke ant mein vadati ya return-type expression \
                       na likho; assertions nigamana/sadrishya-nigamana se karo.".to_string()
                  ),
              },
              TypeCheckError::InvalidNaamadheyaFormat(msg) => Diagnostic {
                  severity: Severity::Dosha,
                  code: "D090".to_string(),
                  sanskrit_title: "अवैधनामधेयरूप".to_string(),
                  roman_title: "Invalid Naamadheya Format".to_string(),
                  message: msg.clone(),
                  sutra_ref: Some(
                      "Chandogya Upanishad 6.1.4 (Vacharambhana — version-label must have a well-formed shape)".to_string()
                  ),
                  hint: Some(
                      "naamadheya ko MAJOR.MINOR.PATCH format mein likho, e.g. \"1.0.0\" ya \"0.1.0\". \
                       Aoptional pre-release hyphen-segment bhi add kar sakte ho, e.g. \"1.0.0-alpha\"."
                          .to_string()
                  ),
              },
              TypeCheckError::InvalidPackageName => Diagnostic {
                  severity: Severity::Dosha,
                  code: "D095".to_string(),
                  sanskrit_title: "अवैधपैकेजनाम".to_string(),
                  roman_title: "Invalid Package Name".to_string(),
                  message: "Mrittika block ka package name valid ASCII identifier \
                            nahi hai. Sirf letters (a-z, A-Z), digits (0-9), aur \
                            hyphens (-) allowed hain; name letter se start hona \
                            chahiye; consecutive hyphens ya trailing hyphen nahi \
                            chahiye; leading/trailing whitespace nahi chahiye."
                      .to_string(),
                  sutra_ref: Some(
                      "Chandogya Upanishad 6.1.4 (Vacharambhana — substance/identity must have a well-formed name)".to_string()
                  ),
                  hint: Some(
                      "Package name mein sirf letters, digits, aur single hyphens use karo. \
                       Example: \"devvani-core\", \"my-package\", \"a\"."
                          .to_string()
                  ),
              },
                TypeCheckError::SatyaBhedaRequiresMajorBump => Diagnostic {
                    severity: Severity::Dosha,
                    code: "D096".to_string(),
                    sanskrit_title: "सत्यभेदमहत्तरबुंद".to_string(),
                    roman_title: "Satya Bheda Requires Major Bump".to_string(),
                    message: "Satya-bheda (breaking change) declare kar rahe ho \
                              jab MAJOR version >= 1 par ho. Ek breaking change \
                              ka naamadheya mein major-version increment reflect \
                              karna chahiye. Kyunki yeh single-file check hai \
                              (kinhin prior version ki tulna nahi kar sakte), \
                              yahaan MAJOR >= 1 par satya-bheda add karne ka \
                              matlab hai ki naamadheya itself naye major-version \
                              ka pratik hone chahiye."
                        .to_string(),
                    sutra_ref: Some(
                        "Chandogya Upanishad 6.1.4 (Vacharambhana — breaking change requires a new name/version for the substance)".to_string()
                    ),
                    hint: Some(
                        "Naamadheya ka MAJOR component bump karo (e.g. \"2.0.0\" ya \"1.0.0\") \
                         jab satya-bheda add karo."
                            .to_string(),
                    ),
                },
                TypeCheckError::ForeignCallOutsideShraddha { name } => Diagnostic {
                    severity: Severity::Dosha,
                    code: "D100".to_string(),
                    sanskrit_title: "बाह्यधातुश्रद्धाबाह्यः".to_string(),
                    roman_title: "Bahya Dhatu Shraddha Bahyah".to_string(),
                    message: format!(
                        "bahya-dhatu (foreign function) '{}' ko sirf shraddha {{ }} \
                         trust block ke andar hi call kiya ja sakta hai. Shraddha \
                         block ke bahar foreign functions call karna apashastra \
                         (prohibited) hai. Mīmāṃsā vyākhyāna: śraddhā (trust) ka \
                         visheṣa (specific context) avasyaka hai — bahya-dhatu ka \
                         upayoga kevala śraddhā-gata (trusted) context mein hi \
                         valid hai.",
                        name
                    ),
                    sutra_ref: Some(
                        "Mīmāṃsā Sūtra (śraddhā — trust/confidence doctrine for foreign interop)".to_string()
                    ),
                    hint: Some(
                        "Is foreign call ko ek shraddha block ke andar likho: \
                         `shraddha {{ ... <naam>(...) ; }}`."
                            .to_string(),
                    ),
                },
                TypeCheckError::DuplicateExportedDhatu { name } => Diagnostic {
                    severity: Severity::Dosha,
                    code: "D102".to_string(),
                    sanskrit_title: "निर्यातधातुद्वन्द्व".to_string(),
                    roman_title: "Niryata Dhatu Dvandva".to_string(),
                    message: format!(
                        "aptavakya dhatu '{}' do baar define ho raha hai — \
                         exported (aptavakya) dhatu names must be globally \
                         unique within the compilation unit. Rust ke #[no_mangle] \
                         jaise uniqueness requirement ke anusaar, duplicate \
                         exported function names allowed nahi hain.",
                        name
                    ),
                    sutra_ref: Some(
                        "Mīmāṃsā Sūtra (aptavakya — trusted-speech uniqueness doctrine)".to_string()
                    ),
                    hint: Some(
                        "Doosri '{}' exported dhatu ka naam change karo ya \
                         usko non-exported dhatu banado."
                            .to_string(),
                    ),
                },
            }
       }

    pub fn from_parse_error(err: &ParseError) -> Diagnostic {
        match err {
            ParseError::AssertionArgCount { keyword, expected, found, .. } => Diagnostic {
                severity: Severity::Dosha,
                code: "D083".to_string(),
                sanskrit_title: "अङ्ग-अप्राप्ति".to_string(),
                roman_title: "Anga Apraapti".to_string(),
                message: format!(
                    "Assertion '{}' ko exactly {} argument(s) chahiye, \
                     par {} mile. Nyāya vyākhyāna: nigamana \
                     (conclusion) ka siddhānta apraāpta (incomplete) hai.",
                    keyword, expected, found
                ),
                sutra_ref: Some(
                    "Nyāya Sūtra (Nigamana — five-membered syllogism conclusion)".to_string()
                ),
                hint: Some(format!(
                    "'{}' ko sirf {} argument do.",
                    keyword, expected
                )),
            },
            ParseError::TarkaWithoutParikshaa { .. } => Diagnostic {
                severity: Severity::Dosha,
                code: "D084".to_string(),
                sanskrit_title: "तर्कविहिता".to_string(),
                roman_title: "Tarka Vihita".to_string(),
                message: "tarka (hypothetical modifier) sirf parikshaa (test) ke \
                         saath hi upyog hota hai. Tarka binā parikshaa ke \
                         svatantra nahi hai. Nyāya vyākhyāna: tarka = \
                         hypothetical reasoning, which requires a paksha \
                         (subject) to which it is applied."
                    .to_string(),
                sutra_ref: Some(
                    "Nyāya Sūtra (Tarka — hypothetical reasoning doctrine)".to_string()
                ),
                hint: Some(
                    "tarka ko parikshaa ke aage lagao: `tarka parikshaa <name> { ... }`."
                        .to_string(),
                ),
            },
            ParseError::MalformedParikshaa { reason, .. } => Diagnostic {
                severity: Severity::Dosha,
                code: "D085".to_string(),
                sanskrit_title: "परीक्षाविघटन".to_string(),
                roman_title: "Parikshaa Vighatan".to_string(),
                message: format!(
                    "Parikshaa (test) malformed hai: {}. \
                     Nyāya vyākhyāna: parikshaa ka sampuṭa (closure) \
                     aur nāma donon avyavahit (essential) hain.",
                    reason
                ),
                sutra_ref: Some(
                    "Nyāya Sūtra (Parikshaa — five-membered syllogism examination)".to_string()
                ),
                hint: Some(
                    "Sahi syntax me parikshaa likho: `parikshaa <name> { ... }`."
                        .to_string(),
                ),
            },
            ParseError::DuplicateMrittika { span: _ } => Diagnostic {
                severity: Severity::Dosha,
                code: "D093".to_string(),
                sanskrit_title: "द्विमृत्तिकादोषः".to_string(),
                roman_title: "Dvi-Mrittika Dosha".to_string(),
                message: "Ek file mein kevala ek mrittika block hi \
                          upyog ho sakta hai. Duplicate mrittika block \
                          nahi chahiye."
                    .to_string(),
                sutra_ref: Some(
                    "Chandogya Upanishad 6.1.4 (Vacharambhana — one clay, one name)".to_string()
                ),
                hint: Some(
                    "Doosri mrittika block hatao ya uske naam ko \
                      pehle ke block mein jod do."
                        .to_string(),
                ),
            },
            ParseError::MissingNaamadheya { span: _ } => Diagnostic {
                severity: Severity::Dosha,
                code: "D094".to_string(),
                sanskrit_title: "नामधेयाभावः".to_string(),
                roman_title: "Naamadheya Abhava".to_string(),
                message: "Mrittika block ka pehla entry naamadheya \
                          hona chahiye — koi non-empty version string \
                          nahi mila. Vacharambhana doctrine: mrittika \
                          (clay/identity) ke bina naamadheya (name/version) \
                          sthanapanna nahi ho sakta."
                    .to_string(),
                sutra_ref: Some(
                    "Chandogya Upanishad 6.1.4 (Vacharambhana — name depends on substance)".to_string()
                ),
                hint: Some(
                    "Mrittika block ke andar pehli line me \
                      `naamadheya \"<version>\" ;` likho."
                        .to_string(),
                ),
            },
            ParseError::InvalidAptavakyaUsage { found, .. } => Diagnostic {
                severity: Severity::Dosha,
                code: "D097".to_string(),
                sanskrit_title: "अप्तवाक्यविहिता".to_string(),
                roman_title: "Aptavakya Vihita".to_string(),
                message: format!(
                    "aptavakya ka upyog galat hai — iske baad ya toh \
                     ek string literal (extern block ke liye) ya dhatu \
                     keyword (exported function ke liye) aana chahiye. \
                     Found: {:?}. Mimāṃsā vyākhyāna: aptavakya (trusted \
                     statement) ko uske visheṣa (specific form) ke saath \
                     hi upyog kiya ja sakta hai.",
                    found
                ),
                sutra_ref: Some(
                    "Mīmāṃsā Sūtra (aptavakya — trusted-speech doctrine)".to_string()
                ),
                hint: Some(
                    "aptavakya ko sahi syntax mein likho: `aptavakya \"C\" { ... }` \
                     ya `aptavakya dhatu <naam> { ... }`."
                        .to_string(),
                ),
            },
            ParseError::BahyaDhatuOutsideAptavakya { .. } => Diagnostic {
                severity: Severity::Dosha,
                code: "D098".to_string(),
                sanskrit_title: "बाह्यधातुबहिर्निवेशः".to_string(),
                roman_title: "Bahya Dhatu Bahirniveshah".to_string(),
                message: "bahya-dhatu (foreign function signature) sirf \
                          aptavakya \"<abi>\" block ke andar hi valid hai. \
                          Yeh block ke bahar nahi likh sakte. Mimāṃsā \
                          vyākhyāna: bahya (external/foreign) ka visheṣa \
                          (specific context) avasyaka hai."
                    .to_string(),
                sutra_ref: Some(
                    "Mīmāṃsā Sūtra (bahya-dhatu — foreign-function context doctrine)".to_string()
                ),
                hint: Some(
                    "bahya-dhatu ko aptavakya block ke andar hi likho: \
                     `aptavakya \"C\" { bahya-dhatu <naam>(...) <type> ; }`."
                        .to_string(),
                ),
            },
            ParseError::BahyaDhatuMustNotHaveBody { .. } => Diagnostic {
                severity: Severity::Dosha,
                code: "D099".to_string(),
                sanskrit_title: "बाह्यधातुशरीरवर्जनम्".to_string(),
                roman_title: "Bahya Dhatu Sharira Varjanam".to_string(),
                message: "bahya-dhatu signature-only declaration hai — \
                          iske andar body nahi ho sakti. Agar aapne by \
                          mistake `{` laga diya hai toh usko `;` se \
                          replace karo. Mimāṃsā vyākhyāna: bahya-dhatu \
                          ka rupa (form) kevala signature tak hi simit \
                          hai, implementation nahi."
                    .to_string(),
                sutra_ref: Some(
                    "Mīmāṃsā Sūtra (bahya-dhatu — signature-only declaration doctrine)".to_string()
                ),
                hint: Some(
                    "bahya-dhatu declaration ko `;` se terminate karo: \
                     `bahya-dhatu <naam>(<params>) <return_type> ;`."
                        .to_string(),
                ),
            },
            _ => Diagnostic {
                severity: Severity::Dosha,
                code: "D009".to_string(),
                sanskrit_title: "वाक्य-संरचना-दोष".to_string(),
                roman_title: "Vakya Sanrachna Dosha".to_string(),
                message: format!("SOV krama galat hai: {}", err),
                sutra_ref: Some("2.1.1".to_string()),
                hint: Some(
                    "Devvani SOV order follow karo: Kartā Karma Kriyā.".to_string(),
                ),
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

    #[test]
    fn test_from_type_error_phala_visamgati() {
        let err = TypeCheckError::PhalaVisamgati {
            expected: devvani_typesystem::DevvaniType::Subject("Purnaank".to_string()),
            found: devvani_typesystem::DevvaniType::Vaak,
        };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D061");
        assert!(diag.display().contains("Phala Visamgati"));
        assert!(diag.sanskrit_title.contains("फलविसंगति"));
        assert!(diag.sutra_ref.unwrap().contains("Charaka Samhita"));
    }

    #[test]
    fn test_from_type_error_nidana_aparichaya() {
        let err = TypeCheckError::NidanaAparichaya;
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D062");
        assert!(diag.display().contains("Nidana Aparichaya"));
        assert!(diag.sanskrit_title.contains("निदानअपरिचय"));
        assert!(diag.sutra_ref.unwrap().contains("Charaka Samhita"));
    }

    #[test]
    fn test_from_type_error_pancaka_avishishtata() {
        let err = TypeCheckError::PancakaAvishishtata;
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D063");
        assert!(diag.display().contains("Pancaka Avishishtata"));
        assert!(diag.sanskrit_title.contains("पञ्चकअविशिष्टता"));
        assert!(diag.sutra_ref.unwrap().contains("Charaka Samhita"));
    }

    #[test]
    fn test_from_type_error_samprāpti_ayogyatā() {
        let err = TypeCheckError::SamprāptiAyogyatā;
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D064");
        assert!(diag.display().contains("Samprāpti Ayogyatā"));
        assert!(diag.sanskrit_title.contains("सम्प्राप्तिअयोग्यता"));
        assert!(diag.sutra_ref.unwrap().contains("Charaka Samhita"));
    }

    #[test]
    fn test_from_type_error_dosha_asangati() {
        let err = TypeCheckError::DoshaAsangati {
            expected: devvani_typesystem::DevvaniType::Subject("Dashaamsha".to_string()),
            found: devvani_typesystem::DevvaniType::Vaak,
        };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D065");
        assert!(diag.display().contains("Dosha Asangati"));
        assert!(diag.sanskrit_title.contains("दोषअसङ्गति"));
        assert!(diag.sutra_ref.unwrap().contains("Charaka Samhita"));
    }

    #[test]
    fn test_from_type_error_phala_sandarbha_abhava() {
        let err = TypeCheckError::PhalaSandarbhaAbhava;
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D066");
        assert!(diag.display().contains("Phala Sandarbha Abhava"));
        assert!(diag.sanskrit_title.contains("फलसन्दर्भअभाव"));
        assert!(diag.sutra_ref.unwrap().contains("Charaka Samhita"));
    }

    #[test]
    fn test_from_type_error_svatva_bhanga_d067() {
        let err = TypeCheckError::SvatvaBhanga {
            name: "ramah".to_string(),
        };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D067");
        assert!(diag.display().contains("Svatva Bhanga"));
        assert!(diag.sanskrit_title.contains("स्वत्वभङ्ग"));
        assert!(diag.message.contains("ramah"));
    }

    #[test]
    fn test_from_type_error_adhikara_dvandva_d068() {
        let err = TypeCheckError::AdhikaraDvandva {
            name: "ramah".to_string(),
        };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D068");
        assert!(diag.display().contains("Adhikara Dvandva"));
        assert!(diag.sanskrit_title.contains("अधिकारद्वन्द्व"));
        assert!(diag.message.contains("ramah"));
    }

    #[test]
    fn test_from_type_error_kshaya_anantara_upayoga_d069() {
        let err = TypeCheckError::KshayaAnantaraUpayoga {
            name: "ramah".to_string(),
        };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D069");
        assert!(diag.display().contains("Kshaya Anantara Upayoga"));
        assert!(diag.sanskrit_title.contains("क्षयानन्तरउपयोग"));
        assert!(diag.message.contains("ramah"));
    }

    #[test]
    fn test_from_type_error_vikara_adhikara_dvaya_d070() {
        let err = TypeCheckError::VikaraAdhikaraDvaya {
            name: "ramah".to_string(),
        };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D070");
        assert!(diag.display().contains("Vikara Adhikara Dvaya"));
        assert!(diag.sanskrit_title.contains("विकारअधिकारद्वय"));
        assert!(diag.message.contains("ramah"));
    }

    #[test]
    fn test_from_type_error_samanya_anishchita_dvandva_d071() {
        let err = TypeCheckError::SamanyaAnishchitaDvandva {
            name: "Yugala".to_string(),
            param_name: "T".to_string(),
            found_type: devvani_typesystem::DevvaniType::Vaak,
            previous_type: devvani_typesystem::DevvaniType::Subject("Purnaank".to_string()),
        };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D071");
        assert!(diag.display().contains("Samanya Anishchita Dvandva"));
        assert!(diag.sanskrit_title.contains("सामान्यअनिश्चितद्वन्द्व"));
        assert!(diag.message.contains("Yugala"));
        assert!(diag.message.contains("T"));
        assert!(diag.sutra_ref.unwrap().contains("Vaiśeṣika"));
    }

    #[test]
    fn test_from_type_error_samanya_aniyata_d072() {
        let err = TypeCheckError::SamanyaAniyata {
            name: "avaghataka".to_string(),
            param_name: "T".to_string(),
        };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D072");
        assert!(diag.display().contains("Samanya Aniyata"));
        assert!(diag.sanskrit_title.contains("सामान्यअनियता"));
        assert!(diag.message.contains("avaghataka"));
        assert!(diag.message.contains("T"));
        assert!(diag.sutra_ref.unwrap().contains("Vaiśeṣika"));
    }

    #[test]
    fn test_from_type_error_anumana_viphalata_d073() {
        let err = TypeCheckError::AnumanaViphalata;
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D073");
        assert!(diag.display().contains("Anumana Viphalata"));
        assert!(diag.sanskrit_title.contains("अनुमानविफलता"));
        assert!(diag.message.contains("Anumāṇa"));
        assert!(diag.sutra_ref.unwrap().contains("Nyāya"));
    }

    #[test]
    fn test_from_type_error_anumana_samgati_bhanga_d074() {
        let err = TypeCheckError::AnumanaSamgatiBhanga;
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D074");
        assert!(diag.display().contains("Anumana Samgati Bhanga"));
        assert!(diag.sanskrit_title.contains("अनुमानसंगतिभङ्ग"));
        assert!(diag.message.contains("Anumāṇa"));
        assert!(diag.sutra_ref.unwrap().contains("Nyāya"));
    }

    #[test]
    fn test_from_type_error_prapti_aprayukta_d075() {
        let err = TypeCheckError::PraptiAprayukta {
            found: devvani_typesystem::DevvaniType::Vaak,
        };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D075");
        assert!(diag.display().contains("Prapti Aprayukta"));
        assert!(diag.sanskrit_title.contains("प्राप्त्यप्रयुक्त"));
        assert!(diag.message.contains("Samyoga"));
        assert!(diag.sutra_ref.unwrap().contains("Nyaya-Vaisheshika"));
    }

    #[test]
    fn test_from_type_error_duta_bhej_aprayukta_d076() {
        let err = TypeCheckError::DutaBhejAprayukta {
            found: devvani_typesystem::DevvaniType::Vaak,
        };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D076");
        assert!(diag.display().contains("Duta Bhej Aprayukta"));
        assert!(diag.sanskrit_title.contains("दूतभेजअप्रयुक्त"));
        assert!(diag.message.contains("DutaBhejaka"));
        assert!(diag.sutra_ref.unwrap().contains("Sandesha-kavya"));
    }

    #[test]
    fn test_from_type_error_duta_grahan_aprayukta_d077() {
        let err = TypeCheckError::DutaGrahanAprayukta {
            found: devvani_typesystem::DevvaniType::Vaak,
        };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D077");
        assert!(diag.display().contains("Duta Grahan Aprayukta"));
        assert!(diag.sanskrit_title.contains("दूतग्रहणअप्रयुक्त"));
        assert!(diag.message.contains("DutaGrahaka"));
        assert!(diag.sutra_ref.unwrap().contains("Sandesha-kavya"));
    }

    #[test]
    fn test_from_type_error_manas_aprayukta_d078() {
        let err = TypeCheckError::ManasAprayukta {
            found: devvani_typesystem::DevvaniType::Vaak,
        };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D078");
        assert!(diag.display().contains("Manas Aprayukta"));
        assert!(diag.sanskrit_title.contains("मनसअप्रयुक्त"));
        assert!(diag.message.contains("Manas"));
        assert!(diag.sutra_ref.unwrap().contains("Nyaya Sutra"));
    }

    #[test]
    fn test_from_type_error_dhara_vinyasa_asangati_d079() {
        let err = TypeCheckError::DharaVinyasaAsangati {
            found: devvani_typesystem::DevvaniType::Subject("Purnaank".to_string()),
        };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D079");
        assert!(diag.display().contains("Dhara Vinyasa Asangati"));
        assert!(diag.sanskrit_title.contains("धाराविन्यासासंगति"));
        assert!(diag.message.contains("Duta"));
        assert!(diag.sutra_ref.unwrap().contains("Nyaya Sutra"));
    }

    #[test]
    fn test_from_type_error_parinama_asangati_d080() {
        let err = TypeCheckError::ParinamaAsangati {
            stage: 1,
            expected: devvani_typesystem::DevvaniType::Subject("Purnaank".to_string()),
            found: devvani_typesystem::DevvaniType::Subject("Dashaamsha".to_string()),
        };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D080");
        assert!(diag.display().contains("Parinama Asangati"));
        assert!(diag.sanskrit_title.contains("परिणामासंगति"));
        assert!(diag.message.contains("stage 1"));
        assert!(diag.sutra_ref.unwrap().contains("pariṇāma-vāda"));
    }

    #[test]
    fn test_from_type_error_parinama_shunya_d081() {
        let err = TypeCheckError::ParinamaShunya;
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D081");
        assert!(diag.display().contains("Parinama Shunyata"));
        assert!(diag.sanskrit_title.contains("परिणामशून्यता"));
        assert!(diag.message.contains("pariṇāma []"));
        assert!(diag.sutra_ref.unwrap().contains("pariṇāma-vāda"));
    }

    #[test]
    fn test_from_type_error_parinama_dosha_vaisamya_d082() {
        let err = TypeCheckError::ParinamaDoshaVaisamya {
            error_a: devvani_typesystem::DevvaniType::Vaak,
            error_b: devvani_typesystem::DevvaniType::Subject("Dashaamsha".to_string()),
        };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D082");
        assert!(diag.display().contains("Parinama Dosha Vaisamya"));
        assert!(diag.sanskrit_title.contains("परिणामदोषवैषम्य"));
        assert!(diag.message.contains("error types asangat hain"));
        assert!(diag.sutra_ref.unwrap().contains("pariṇāma-vāda"));
    }

    // --- PARIṬṢĀ (TESTING) DIAGNOSTICS ---

    #[test]
    fn test_from_parse_error_assertion_arg_count_d083() {
        let err = ParseError::AssertionArgCount {
            keyword: "nigamana".to_string(),
            expected: 1,
            found: 3,
            span: devvani_lexer::token::Span { line: 1, col: 1, len: 1 },
        };
        let diag = DiagnosticEngine::from_parse_error(&err);
        assert_eq!(diag.code, "D083");
        assert!(diag.display().contains("Anga Apraapti"));
        assert!(diag.sanskrit_title.contains("अङ्ग-अप्राप्ति"));
        assert!(diag.message.contains("nigamana"));
        assert!(diag.message.contains("1"));
        assert!(diag.message.contains("3"));
        assert!(diag.sutra_ref.unwrap().contains("Nyāya Sūtra"));
    }

    #[test]
    fn test_from_parse_error_tarka_without_parikshaa_d084() {
        let err = ParseError::TarkaWithoutParikshaa {
            span: devvani_lexer::token::Span { line: 1, col: 1, len: 1 },
        };
        let diag = DiagnosticEngine::from_parse_error(&err);
        assert_eq!(diag.code, "D084");
        assert!(diag.display().contains("Tarka Vihita"));
        assert!(diag.sanskrit_title.contains("तर्कविहिता"));
        assert!(diag.message.contains("tarka"));
        assert!(diag.sutra_ref.unwrap().contains("Nyāya Sūtra"));
    }

    #[test]
    fn test_from_parse_error_malformed_parikshaa_d085() {
        let err = ParseError::MalformedParikshaa {
            reason: "missing name".to_string(),
            span: devvani_lexer::token::Span { line: 1, col: 1, len: 1 },
        };
        let diag = DiagnosticEngine::from_parse_error(&err);
        assert_eq!(diag.code, "D085");
        assert!(diag.display().contains("Parikshaa Vighatan"));
        assert!(diag.sanskrit_title.contains("परीक्षाविघटन"));
        assert!(diag.message.contains("missing name"));
        assert!(diag.sutra_ref.unwrap().contains("Nyāya Sūtra"));
    }

    // --- PARIṬṢĀ (TESTING) TYPE DIAGNOSTICS ---

    #[test]
    fn test_from_type_error_nigamana_not_bool_d086() {
        let err = TypeCheckError::NigamanaNotBool {
            found: devvani_typesystem::DevvaniType::Subject("Purnaank".to_string()),
        };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D086");
        assert!(diag.display().contains("Nigamana Abhinna Prakaar"));
        assert!(diag.sanskrit_title.contains("निगमनाभिन्नप्रकार"));
        assert!(diag.message.contains("Bool"));
        assert!(diag.sutra_ref.unwrap().contains("Nyāya Sūtra"));
    }

    #[test]
    fn test_from_type_error_sadrishya_nigamana_mismatched_types_d087() {
        let err = TypeCheckError::SadrishyaNigamanaMismatchedTypes {
            left: devvani_typesystem::DevvaniType::Subject("Purnaank".to_string()),
            right: devvani_typesystem::DevvaniType::Vaak,
        };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D087");
        assert!(diag.display().contains("Sadrishya Nigamana Vaisamya"));
        assert!(diag.sanskrit_title.contains("सादृश्यनिगमनवैषम्य"));
        assert!(diag.message.contains("Purnaank"));
        assert!(diag.message.contains("Vaak"));
        assert!(diag.sutra_ref.unwrap().contains("Nyāya Sūtra"));
    }

    #[test]
    fn test_from_type_error_sadrishya_nigamana_not_equality_comparable_d088() {
        let err = TypeCheckError::SadrishyaNigamanaNotEqualityComparable {
            ty: devvani_typesystem::DevvaniType::Unknown,
        };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D088");
        assert!(diag.display().contains("Sadrishya Nigamana Asamartha"));
        assert!(diag.sanskrit_title.contains("सादृश्यनिगमनासमर्थ"));
        assert!(diag.message.contains("Unknown"));
        assert!(diag.sutra_ref.unwrap().contains("Nyāya Sūtra"));
    }

    #[test]
    fn test_from_type_error_parikshaa_body_not_unit_d089() {
        let err = TypeCheckError::ParikshaaBodyNotUnit;
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D089");
        assert!(diag.display().contains("Parikshaa Sharira Vaisamya"));
        assert!(diag.sanskrit_title.contains("परीक्षाशरीरवैषम्य"));
        assert!(diag.message.contains("unit"));
        assert!(diag.sutra_ref.unwrap().contains("Nyāya Sūtra"));
    }

    #[test]
    fn test_from_type_error_invalid_naamadheya_format_d090() {
        let err = TypeCheckError::InvalidNaamadheyaFormat(
            "naamadheya must have exactly three dot-separated numeric components (MAJOR.MINOR.PATCH)".to_string(),
        );
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D090");
        assert!(diag.display().contains("Invalid Naamadheya Format"));
        assert!(diag.sanskrit_title.contains("अवैधनामधेयरूप"));
        assert!(diag.message.contains("MAJOR.MINOR.PATCH"));
        assert!(diag.sutra_ref.unwrap().contains("Vacharambhana"));
    }

    #[test]
    fn test_from_type_error_invalid_package_name_d095() {
        let err = TypeCheckError::InvalidPackageName;
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D095");
        assert!(diag.display().contains("Invalid Package Name"));
        assert!(diag.sanskrit_title.contains("अवैधपैकेजनाम"));
        assert!(diag.message.contains("ASCII identifier"));
        assert!(diag.sutra_ref.unwrap().contains("Vacharambhana"));
    }

    #[test]
    fn test_from_type_error_satya_bheda_requires_major_bump_d096() {
        let err = TypeCheckError::SatyaBhedaRequiresMajorBump;
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D096");
        assert!(diag.display().contains("Satya Bheda Requires Major Bump"));
        assert!(diag.sanskrit_title.contains("सत्यभेदमहत्तरबुंद"));
        assert!(diag.message.contains("MAJOR"));
        assert!(diag.sutra_ref.unwrap().contains("Vacharambhana"));
    }

    // --- D100: ForeignCallOutsideShraddha ---

    #[test]
    fn test_from_type_error_foreign_call_outside_shraddha_d100() {
        let err = TypeCheckError::ForeignCallOutsideShraddha {
            name: "foreign_puts".to_string(),
        };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D100");
        assert!(diag.display().contains("Bahya Dhatu Shraddha Bahyah"));
        assert!(diag.sanskrit_title.contains("बाह्यधातुश्रद्धाबाह्यः"));
        assert!(diag.message.contains("foreign_puts"));
        assert!(diag.message.contains("shraddha"));
        assert!(diag.sutra_ref.unwrap().contains("śraddhā"));
    }

    // --- D102: DuplicateExportedDhatu ---

    #[test]
    fn test_from_type_error_duplicate_exported_dhatu_d102() {
        let err = TypeCheckError::DuplicateExportedDhatu {
            name: "shared_export".to_string(),
        };
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D102");
        assert!(diag.display().contains("Niryata Dhatu Dvandva"));
        assert!(diag.sanskrit_title.contains("निर्यातधातुद्वन्द्व"));
        assert!(diag.message.contains("shared_export"));
        assert!(diag.message.contains("globally unique"));
        assert!(diag.sutra_ref.unwrap().contains("aptavakya"));
    }
}
