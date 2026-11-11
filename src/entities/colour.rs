//! Entity: a colour of the sRGB space, as a stylesheet writes it in hexadecimal.

use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Colour {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Colour {
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Colour { red, green, blue }
    }

    /// `#rrggbb` or `#rgb`, in capitals or small letters; `None` for anything else.
    pub fn from_hex(text: &str) -> Option<Colour> {
        let digits = text.strip_prefix('#')?;
        if !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return None;
        }
        let value = |part: &str| u8::from_str_radix(part, 16).ok();
        match digits.len() {
            6 => Some(Colour::new(
                value(&digits[0..2])?,
                value(&digits[2..4])?,
                value(&digits[4..6])?,
            )),
            3 => {
                let double = |index: usize| value(&digits[index..=index]).map(|digit| digit * 17);
                Some(Colour::new(double(0)?, double(1)?, double(2)?))
            }
            _ => None,
        }
    }
}

/// `#rrggbb`, in small letters.
impl Display for Colour {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(
            formatter,
            "#{:02x}{:02x}{:02x}",
            self.red, self.green, self.blue
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn six_or_three_digits_after_a_hash() {
        assert_eq!(Colour::from_hex("#0B5394"), Some(Colour::new(11, 83, 148)));
        assert_eq!(Colour::from_hex("#fa0"), Some(Colour::new(255, 170, 0)));
        assert_eq!(Colour::new(11, 83, 148).to_string(), "#0b5394");
    }

    #[test]
    fn anything_else_is_not_a_colour() {
        for text in [
            "", "#", "0b5394", "#0b539", "#0b53944", "#ggg", "blue", "#+1+", "#é1",
        ] {
            assert_eq!(Colour::from_hex(text), None, "{text:?}");
        }
    }
}
