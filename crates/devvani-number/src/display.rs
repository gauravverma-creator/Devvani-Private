//! Devanagari display implementation
use crate::{DevvaniFloat, DevvaniInt};
use std::fmt;

const DEVA_CHARS: [char; 10] = ['०', '१', '२', '३', '४', '५', '६', '७', '८', '९'];

impl fmt::Display for DevvaniInt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_negative {
            write!(f, "-")?;
        }
        if self.value == 0 {
            return write!(f, "०");
        }

        let mut digits = ['\0'; 20];
        let mut n = self.value as u64;
        let mut len = 0;
        while n > 0 {
            digits[len] = DEVA_CHARS[(n % 10) as usize];
            n /= 10;
            len += 1;
        }
        for i in (0..len).rev() {
            write!(f, "{}", digits[i])?;
        }
        Ok(())
    }
}

impl fmt::Display for DevvaniFloat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.value.is_nan() {
            return write!(f, "अपरिभाषित");
        }
        if self.value.is_infinite() {
            return write!(f, "अनंत");
        }
        if self.value < 0.0 {
            write!(f, "-")?;
        }

        let val = self.value.abs();
        let int_part = val.floor() as u64;
        let frac_part = val - val.floor();

        // Integer part
        if int_part == 0 {
            write!(f, "०")?;
        } else {
            let mut digits = ['\0'; 20];
            let mut n = int_part;
            let mut len = 0;
            while n > 0 {
                digits[len] = DEVA_CHARS[(n % 10) as usize];
                n /= 10;
                len += 1;
            }
            for i in (0..len).rev() {
                write!(f, "{}", digits[i])?;
            }
        }

        write!(f, ".")?;

        // Fractional part (up to 15 digits)
        let mut f_val = frac_part;
        if f_val == 0.0 {
            write!(f, "०")?;
        } else {
            for _ in 0..15 {
                f_val *= 10.0;
                let digit = f_val.floor() as usize;
                write!(f, "{}", DEVA_CHARS[digit])?;
                f_val -= f_val.floor();
                if f_val < 1e-15 {
                    break;
                }
            }
        }
        Ok(())
    }
}
