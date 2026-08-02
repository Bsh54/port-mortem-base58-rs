use std::sync::LazyLock;

use crate::error::AlphabetError;

pub const ALPHABET_SIZE: usize = 58;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alphabet {
    encode: [u8; ALPHABET_SIZE],
    decode: [i8; 128],
}

impl Alphabet {
    pub fn new(s: &str) -> Result<Self, AlphabetError> {
        let bytes = s.as_bytes();
        if bytes.len() != ALPHABET_SIZE {
            return Err(AlphabetError::WrongLength(bytes.len()));
        }

        let mut encode = [0u8; ALPHABET_SIZE];
        let mut decode = [-1i8; 128];

        for (i, &b) in bytes.iter().enumerate() {
            if b > 127 {
                return Err(AlphabetError::NonAscii(b));
            }
            if decode[b as usize] != -1 {
                return Err(AlphabetError::Duplicate(b));
            }
            encode[i] = b;
            decode[b as usize] = i as i8;
        }

        Ok(Alphabet { encode, decode })
    }

    #[inline]
    pub(crate) fn encode_digit(&self, value: u64) -> u8 {
        self.encode[value as usize]
    }

    #[inline]
    pub(crate) fn zero_char(&self) -> u8 {
        self.encode[0]
    }

    #[inline]
    pub(crate) fn decode_byte(&self, ch: u8) -> i8 {
        self.decode[ch as usize]
    }
}

pub static BTC_ALPHABET: LazyLock<Alphabet> = LazyLock::new(|| {
    Alphabet::new("123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz")
        .expect("BTC alphabet is valid")
});

pub static FLICKR_ALPHABET: LazyLock<Alphabet> = LazyLock::new(|| {
    Alphabet::new("123456789abcdefghijkmnopqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ")
        .expect("Flickr alphabet is valid")
});
