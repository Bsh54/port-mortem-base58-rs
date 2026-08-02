#![forbid(unsafe_code)]

mod alphabet;
mod error;
mod fast;
mod trivial;

pub use alphabet::{Alphabet, BTC_ALPHABET, FLICKR_ALPHABET};
pub use error::{AlphabetError, DecodeError};

pub fn encode(bin: &[u8]) -> String {
    fast::encode(bin, &BTC_ALPHABET)
}

pub fn encode_alphabet(bin: &[u8], alphabet: &Alphabet) -> String {
    fast::encode(bin, alphabet)
}

pub fn decode(input: impl AsRef<[u8]>) -> Result<Vec<u8>, DecodeError> {
    fast::decode(input.as_ref(), &BTC_ALPHABET)
}

pub fn decode_alphabet(
    input: impl AsRef<[u8]>,
    alphabet: &Alphabet,
) -> Result<Vec<u8>, DecodeError> {
    fast::decode(input.as_ref(), alphabet)
}

pub fn fast_encode(bin: &[u8]) -> String {
    fast::encode(bin, &BTC_ALPHABET)
}

pub fn fast_encode_alphabet(bin: &[u8], alphabet: &Alphabet) -> String {
    fast::encode(bin, alphabet)
}

pub fn fast_decode(input: impl AsRef<[u8]>) -> Result<Vec<u8>, DecodeError> {
    fast::decode(input.as_ref(), &BTC_ALPHABET)
}

pub fn fast_decode_alphabet(
    input: impl AsRef<[u8]>,
    alphabet: &Alphabet,
) -> Result<Vec<u8>, DecodeError> {
    fast::decode(input.as_ref(), alphabet)
}

pub fn trivial_encode(bin: &[u8]) -> String {
    trivial::encode(bin, &BTC_ALPHABET)
}

pub fn trivial_encode_alphabet(bin: &[u8], alphabet: &Alphabet) -> String {
    trivial::encode(bin, alphabet)
}

pub fn trivial_decode(input: impl AsRef<[u8]>) -> Result<Vec<u8>, DecodeError> {
    trivial::decode(input.as_ref(), &BTC_ALPHABET)
}

pub fn trivial_decode_alphabet(
    input: impl AsRef<[u8]>,
    alphabet: &Alphabet,
) -> Result<Vec<u8>, DecodeError> {
    trivial::decode(input.as_ref(), alphabet)
}
