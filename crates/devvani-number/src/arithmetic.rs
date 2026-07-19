//! Overloaded operators for Devvani types
use crate::{DevvaniFloat, DevvaniInt, NumberError};
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

impl Add for DevvaniInt {
    type Output = Result<DevvaniInt, NumberError>;
    fn add(self, rhs: Self) -> Self::Output {
        self.checked_add(&rhs)
    }
}

impl Sub for DevvaniInt {
    type Output = Result<DevvaniInt, NumberError>;
    fn sub(self, rhs: Self) -> Self::Output {
        self.checked_sub(&rhs)
    }
}

impl Mul for DevvaniInt {
    type Output = Result<DevvaniInt, NumberError>;
    fn mul(self, rhs: Self) -> Self::Output {
        self.checked_mul(&rhs)
    }
}

impl Div for DevvaniInt {
    type Output = Result<DevvaniFloat, NumberError>;
    fn div(self, rhs: Self) -> Self::Output {
        let b = rhs.to_i64();
        if b == 0 {
            return Err(NumberError::ShoolyVibhaajan);
        }
        Ok(DevvaniFloat::from_f64(self.to_i64() as f64 / b as f64))
    }
}

impl Rem for DevvaniInt {
    type Output = Result<DevvaniInt, NumberError>;
    fn rem(self, rhs: Self) -> Self::Output {
        let b = rhs.to_i64();
        if b == 0 {
            return Err(NumberError::ShoolyVibhaajan);
        }
        Ok(DevvaniInt::from_i64(self.to_i64() % b))
    }
}

impl Neg for DevvaniInt {
    type Output = DevvaniInt;
    fn neg(self) -> Self::Output {
        DevvaniInt {
            value: self.value,
            is_negative: !self.is_negative,
        }
    }
}

impl Add for DevvaniFloat {
    type Output = DevvaniFloat;
    fn add(self, rhs: Self) -> Self::Output {
        DevvaniFloat::from_f64(self.value + rhs.value)
    }
}

impl Sub for DevvaniFloat {
    type Output = DevvaniFloat;
    fn sub(self, rhs: Self) -> Self::Output {
        DevvaniFloat::from_f64(self.value - rhs.value)
    }
}

impl Mul for DevvaniFloat {
    type Output = DevvaniFloat;
    fn mul(self, rhs: Self) -> Self::Output {
        DevvaniFloat::from_f64(self.value * rhs.value)
    }
}

impl Div for DevvaniFloat {
    type Output = Result<DevvaniFloat, NumberError>;
    fn div(self, rhs: Self) -> Self::Output {
        if rhs.value == 0.0 {
            return Err(NumberError::ShoolyVibhaajan);
        }
        Ok(DevvaniFloat::from_f64(self.value / rhs.value))
    }
}

pub fn sqrt(n: &DevvaniFloat) -> Result<DevvaniFloat, NumberError> {
    if n.value < 0.0 {
        return Err(NumberError::NegativeSqrt);
    }
    Ok(DevvaniFloat::from_f64(n.value.sqrt()))
}

pub fn pow_int(base: &DevvaniInt, exp: &DevvaniInt) -> Result<DevvaniInt, NumberError> {
    let e = exp.to_i64();
    if e < 0 {
        return Err(NumberError::ValueError);
    }
    base.checked_pow(e as u32)
}

pub fn pi() -> DevvaniFloat {
    DevvaniFloat::from_f64(std::f64::consts::PI)
}
pub fn e() -> DevvaniFloat {
    DevvaniFloat::from_f64(std::f64::consts::E)
}
