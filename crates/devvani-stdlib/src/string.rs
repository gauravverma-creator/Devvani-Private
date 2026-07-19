//! वाक् (Vaak) String Dhatus — Low-level C-ABI string operations
//!
//! These functions are called from LLVM IR and follow the Kāraka ownership model:
//! - Yoga/Khanda return *mut i8 — caller owns (Karta)
//! - Parimana takes *const i8 — borrow only (Karaṇa), no free needed
//! - Mukta frees Karta-owned strings

use std::ffi::{c_char, CStr, CString};
use std::ptr;

/// __devvani_vaak_yoga(left: *const i8, right: *const i8) -> *mut i8
/// Concatenates two strings. Caller owns the result (Karta).
#[no_mangle]
pub extern "C" fn __devvani_vaak_yoga(left: *const c_char, right: *const c_char) -> *mut c_char {
    unsafe {
        let left_str = CStr::from_ptr(left).to_str().unwrap_or("");
        let right_str = CStr::from_ptr(right).to_str().unwrap_or("");
        let result = format!("{}{}", left_str, right_str);
        CString::new(result)
            .map(|cs| cs.into_raw())
            .unwrap_or(ptr::null_mut())
    }
}

/// __devvani_vaak_parimana(s: *const i8) -> i64
/// Returns byte length of string. Does not transfer ownership (Karaṇa borrow).
#[no_mangle]
pub extern "C" fn __devvani_vaak_parimana(s: *const c_char) -> i64 {
    unsafe { CStr::from_ptr(s).to_bytes().len() as i64 }
}

/// __devvani_vaak_khanda(s: *const i8, start: i64, end: i64) -> *mut i8
/// Returns substring [start..end]. Caller owns result (Karta).
/// Returns empty string if indices out of bounds (safe, no panic).
#[no_mangle]
pub extern "C" fn __devvani_vaak_khanda(s: *const c_char, start: i64, end: i64) -> *mut c_char {
    unsafe {
        let str_slice = CStr::from_ptr(s).to_str().unwrap_or("");
        let len = str_slice.len() as i64;
        let clamped_start = start.max(0).min(len);
        let clamped_end = end.max(0).min(len);
        let actual_start = clamped_start.min(clamped_end);
        let actual_end = clamped_end;
        let result = &str_slice[actual_start as usize..actual_end as usize];
        CString::new(result)
            .map(|cs| cs.into_raw())
            .unwrap_or(ptr::null_mut())
    }
}

/// __devvani_vaak_mukta(s: *mut i8)
/// Frees a Karta-owned string returned by yoga/khanda.
#[no_mangle]
pub extern "C" fn __devvani_vaak_mukta(s: *mut c_char) {
    unsafe {
        if s.is_null() {
            return;
        }
        let _ = CString::from_raw(s);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yoga_basic() {
        let left = CString::new("नमस्ते").unwrap();
        let right = CString::new(" देवाणी").unwrap();
        let result = unsafe {
            let ptr = __devvani_vaak_yoga(left.as_ptr(), right.as_ptr());
            if ptr.is_null() {
                "__devvani_vaak_yoga failed".to_string()
            } else {
                let s = CStr::from_ptr(ptr).to_str().unwrap().to_string();
                __devvani_vaak_mukta(ptr);
                s
            }
        };
        assert_eq!(result, "नमस्ते देवाणी");
    }

    #[test]
    fn test_yoga_empty() {
        let left = CString::new("").unwrap();
        let right = CString::new("abc").unwrap();
        let result = unsafe {
            let ptr = __devvani_vaak_yoga(left.as_ptr(), right.as_ptr());
            if ptr.is_null() {
                "__devvani_vaak_yoga failed".to_string()
            } else {
                let s = CStr::from_ptr(ptr).to_str().unwrap().to_string();
                __devvani_vaak_mukta(ptr);
                s
            }
        };
        assert_eq!(result, "abc");
    }

    #[test]
    fn test_parimana_ascii() {
        let s = CString::new("hello").unwrap();
        let len = __devvani_vaak_parimana(s.as_ptr());
        assert_eq!(len, 5);
    }

    #[test]
    fn test_parimana_empty() {
        let s = CString::new("").unwrap();
        let len = __devvani_vaak_parimana(s.as_ptr());
        assert_eq!(len, 0);
    }

    #[test]
    fn test_khanda_basic() {
        // Byte-based slicing: "deva" -> [0..2] = "de"
        let s = CString::new("deva").unwrap();
        let result = unsafe {
            let ptr = __devvani_vaak_khanda(s.as_ptr(), 0, 2);
            if ptr.is_null() {
                "__devvani_vaak_khanda failed".to_string()
            } else {
                let s = CStr::from_ptr(ptr).to_str().unwrap().to_string();
                __devvani_vaak_mukta(ptr);
                s
            }
        };
        assert_eq!(result, "de");
    }

    #[test]
    fn test_khanda_oob() {
        let s = CString::new("hello").unwrap();
        let result = unsafe {
            // Out of bounds: start > end, beyond length
            let ptr = __devvani_vaak_khanda(s.as_ptr(), 100, 200);
            if ptr.is_null() {
                "__devvani_vaak_khanda failed".to_string()
            } else {
                let s = CStr::from_ptr(ptr).to_str().unwrap().to_string();
                __devvani_vaak_mukta(ptr);
                s
            }
        };
        // OOB returns empty string, no panic
        assert_eq!(result, "");
    }
}
