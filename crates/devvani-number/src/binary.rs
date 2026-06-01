//! Binary conversions and checked math
use crate::{DevvaniInt, DevvaniFloat, NumberError};

impl DevvaniInt {
    pub fn from_devanagari(digits: &[char]) -> Result<Self, NumberError> {
        let mut val: i64 = 0;
        for &ch in digits {
            let digit = devanagari_to_digit(ch)? as i64;
            val = val.checked_mul(10)
                .and_then(|v| v.checked_add(digit))
                .ok_or(NumberError::Overflow)?;
        }
        Ok(DevvaniInt { value: val, is_negative: false })
    }

    pub fn from_i64(n: i64) -> Self {
        DevvaniInt { value: n.abs(), is_negative: n < 0 }
    }

    pub fn to_i64(&self) -> i64 {
        if self.is_negative { -self.value } else { self.value }
    }

    pub fn checked_add(&self, other: &Self) -> Result<Self, NumberError> {
        self.to_i64().checked_add(other.to_i64()).map(Self::from_i64).ok_or(NumberError::Overflow)
    }

    pub fn checked_sub(&self, other: &Self) -> Result<Self, NumberError> {
        self.to_i64().checked_sub(other.to_i64()).map(Self::from_i64).ok_or(NumberError::Overflow)
    }

    pub fn checked_mul(&self, other: &Self) -> Result<Self, NumberError> {
        self.to_i64().checked_mul(other.to_i64()).map(Self::from_i64).ok_or(NumberError::Overflow)
    }

    pub fn checked_pow(&self, exp: u32) -> Result<Self, NumberError> {
        self.to_i64().checked_pow(exp).map(Self::from_i64).ok_or(NumberError::Overflow)
    }
}

impl DevvaniFloat {
    pub fn from_devanagari_parts(integer_digits: &[char], fractional_digits: &[char]) -> Result<Self, NumberError> {
        let mut int_val: f64 = 0.0;
        for &ch in integer_digits {
            int_val = int_val * 10.0 + (devanagari_to_digit(ch)? as f64);
        }

        let mut frac_val: f64 = 0.0;
        let mut divisor: f64 = 10.0;
        for &ch in fractional_digits {
            frac_val += (devanagari_to_digit(ch)? as f64) / divisor;
            divisor *= 10.0;
        }

        Ok(DevvaniFloat { value: int_val + frac_val })
    }

    pub fn from_f64(f: f64) -> Self {
        Self { value: f }
    }

    pub fn to_f64(&self) -> f64 {
        self.value
    }
}

pub const fn devanagari_to_digit(ch: char) -> Result<u8, NumberError> {
    match ch {
        '०' => Ok(0), '१' => Ok(1), '२' => Ok(2), '३' => Ok(3), '४' => Ok(4),
        '५' => Ok(5), '६' => Ok(6), '७' => Ok(7), '८' => Ok(8), '९' => Ok(9),
        _ => Err(NumberError::InvalidDigit(ch)),
    }
}
