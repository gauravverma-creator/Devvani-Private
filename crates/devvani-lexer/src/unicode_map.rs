pub fn normalize_iast(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            'a' if chars.peek() == Some(&'a') => {
                chars.next();
                result.push('ā');
            }
            'i' if chars.peek() == Some(&'i') => {
                chars.next();
                result.push('ī');
            }
            'u' if chars.peek() == Some(&'u') => {
                chars.next();
                result.push('ū');
            }
            's' if chars.peek() == Some(&'h') => {
                chars.next();
                result.push('ś');
            }
            'S' if chars.peek() == Some(&'h') => {
                chars.next();
                result.push('ṣ');
            }
            'M' => result.push('ṃ'),
            'H' => result.push('ḥ'),
            _ => result.push(c),
        }
    }
    result
}

pub fn is_iast_identifier_start(c: char) -> bool {
    (c.is_alphabetic() || "_".contains(c) || is_iast_special(c)) && c != 'ḥ' && c != 'ṃ'
}

pub fn is_iast_identifier_continue(c: char) -> bool {
    (c.is_alphanumeric() || "_".contains(c) || is_iast_special(c)) && c != 'ḥ' && c != 'ṃ'
}

pub fn is_iast_special(c: char) -> bool {
    // Check if character is within Devanagari block (U+0900 to U+097F)
    if (c as u32) >= 0x0900 && (c as u32) <= 0x097F {
        return true;
    }

    match c {
        '\u{0101}' | '\u{012B}' | '\u{016B}' | // ā, ī, ū
        '\u{1E6D}' | '\u{1E0D}' | '\u{1E47}' | // ṭ, ḍ, ṇ
        '\u{015B}' | '\u{1E63}' | // ś, ṣ
        '\u{1E45}' | '\u{00F1}' | // ṅ, ñ
        '\u{1E5B}' | '\u{1E37}' => true,       // r̥, l̥
        _ => false,
    }
}

pub fn normalize_devanagari(_input: &str) -> String {
    _input.to_string()
}
