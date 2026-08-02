//! C-ABI shim so the original Go test suite can run, unmodified, against the
//! Rust port through cgo. This crate is test scaffolding: it is the only place
//! `unsafe` appears, and it is not part of the published port (which keeps
//! `#![forbid(unsafe_code)]`).

use std::slice;

use base58::{Alphabet, DecodeError, BTC_ALPHABET};

fn build_alphabet(alpha: *const u8) -> Option<Alphabet> {
    if alpha.is_null() {
        return None;
    }
    let bytes = unsafe { slice::from_raw_parts(alpha, 58) };
    std::str::from_utf8(bytes)
        .ok()
        .and_then(|s| Alphabet::new(s).ok())
}

fn input_slice<'a>(ptr: *const u8, len: usize) -> &'a [u8] {
    if len == 0 || ptr.is_null() {
        &[]
    } else {
        unsafe { slice::from_raw_parts(ptr, len) }
    }
}

fn into_raw(bytes: Vec<u8>, out: *mut usize) -> *mut u8 {
    let boxed = bytes.into_boxed_slice();
    let len = boxed.len();
    unsafe {
        *out = len;
    }
    Box::into_raw(boxed) as *mut u8
}

fn err_code(e: &DecodeError) -> i32 {
    match e {
        DecodeError::ZeroLengthString => 1,
        DecodeError::InvalidDigit(_) => 2,
        DecodeError::HighBit(_) => 3,
    }
}

#[no_mangle]
pub extern "C" fn b58_encode(
    inp: *const u8,
    in_len: usize,
    alpha: *const u8,
    fast: i32,
    out: *mut usize,
) -> *mut u8 {
    let input = input_slice(inp, in_len);
    let custom = build_alphabet(alpha);
    let alphabet: &Alphabet = custom.as_ref().unwrap_or(&BTC_ALPHABET);
    let encoded = if fast != 0 {
        base58::encode_alphabet(input, alphabet)
    } else {
        base58::trivial_encode_alphabet(input, alphabet)
    };
    into_raw(encoded.into_bytes(), out)
}

#[no_mangle]
pub extern "C" fn b58_decode(
    inp: *const u8,
    in_len: usize,
    alpha: *const u8,
    fast: i32,
    out: *mut usize,
    err: *mut i32,
) -> *mut u8 {
    let input = input_slice(inp, in_len);
    let custom = build_alphabet(alpha);
    let alphabet: &Alphabet = custom.as_ref().unwrap_or(&BTC_ALPHABET);
    let result = if fast != 0 {
        base58::decode_alphabet(input, alphabet)
    } else {
        base58::trivial_decode_alphabet(input, alphabet)
    };
    match result {
        Ok(bytes) => {
            unsafe {
                *err = 0;
            }
            into_raw(bytes, out)
        }
        Err(e) => {
            unsafe {
                *err = err_code(&e);
                *out = 0;
            }
            std::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn b58_alphabet_ok(alpha: *const u8) -> i32 {
    if alpha.is_null() {
        return 0;
    }
    let bytes = unsafe { slice::from_raw_parts(alpha, 58) };
    match std::str::from_utf8(bytes) {
        Ok(s) => Alphabet::new(s).is_ok() as i32,
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn b58_free(ptr: *mut u8, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }
    unsafe {
        let slice = slice::from_raw_parts_mut(ptr, len);
        drop(Box::from_raw(slice as *mut [u8]));
    }
}
