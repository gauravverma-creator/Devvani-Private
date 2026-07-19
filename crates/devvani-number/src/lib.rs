//! Architecture-aware Devvani Number System

pub mod arithmetic;
pub mod binary;
pub mod display;
pub mod platform;

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DevvaniInt {
    pub value: i64,
    pub is_negative: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct DevvaniFloat {
    pub value: f64,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NumberError {
    Overflow,
    ShoolyVibhaajan, // Division by zero
    InvalidDigit(char),
    NegativeSqrt,
    ValueError,
}

impl fmt::Display for NumberError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NumberError::Overflow => write!(f, "अतिप्रवाह (Overflow)"),
            NumberError::ShoolyVibhaajan => write!(f, "शून्य-विभाजन (Division by Zero)"),
            NumberError::InvalidDigit(c) => write!(f, "अमान्य अंक: {}", c),
            NumberError::NegativeSqrt => write!(f, "ऋणात्मक वर्गमूल (Negative Sqrt)"),
            NumberError::ValueError => write!(f, "मान त्रुटि (Value Error)"),
        }
    }
}
