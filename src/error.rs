use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    ZeroLengthString,
    HighBit(u8),
    InvalidDigit(u8),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::ZeroLengthString => f.write_str("zero length string"),
            DecodeError::HighBit(_) => f.write_str("high-bit set on invalid digit"),
            DecodeError::InvalidDigit(b) => {
                write!(f, "invalid base58 digit ({:?})", *b as char)
            }
        }
    }
}

impl std::error::Error for DecodeError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlphabetError {
    WrongLength(usize),
    NonAscii(u8),
    Duplicate(u8),
}

impl fmt::Display for AlphabetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AlphabetError::WrongLength(n) => {
                write!(f, "base58 alphabets must be 58 bytes long, got {n}")
            }
            AlphabetError::NonAscii(b) => {
                write!(f, "alphabet contains a non-ascii byte (0x{b:02x})")
            }
            AlphabetError::Duplicate(b) => {
                write!(f, "alphabet contains a duplicate character ({:?})", *b as char)
            }
        }
    }
}

impl std::error::Error for AlphabetError {}
