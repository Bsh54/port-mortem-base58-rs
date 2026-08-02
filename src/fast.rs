use crate::alphabet::Alphabet;
use crate::error::DecodeError;

const CHUNK_DIGITS: usize = 10;
const CHUNK_BASE: u64 = 430_804_206_899_405_824; // 58^10

const CHUNK_POWERS: [u64; CHUNK_DIGITS + 1] = [
    1,
    58,
    3_364,
    195_112,
    11_316_496,
    656_356_768,
    38_068_692_544,
    2_207_984_167_552,
    128_063_081_718_016,
    7_427_658_739_644_928,
    CHUNK_BASE,
];

pub fn encode(bin: &[u8], alphabet: &Alphabet) -> String {
    if bin.is_empty() {
        return String::new();
    }

    let zcount = bin.iter().take_while(|&&b| b == 0).count();
    if zcount == bin.len() {
        return String::from_utf8(vec![alphabet.zero_char(); zcount])
            .expect("alphabet bytes are ascii");
    }

    let payload = &bin[zcount..];
    let word_count = payload.len().div_ceil(8);

    let mut words = load_base256_words(payload, word_count);
    let mut scratch = vec![0u64; word_count];

    let chunk_estimate = (payload.len() * 555 / 406).div_ceil(CHUNK_DIGITS);
    let mut chunks: Vec<u64> = Vec::with_capacity(chunk_estimate);

    let mut start = 0usize;
    loop {
        let mut remainder = 0u64;
        let mut next_start = words.len();
        for i in start..words.len() {
            let (quotient, rem) = div_wide(remainder, words[i], CHUNK_BASE);
            scratch[i] = quotient;
            remainder = rem;
            if quotient != 0 && next_start == words.len() {
                next_start = i;
            }
        }
        chunks.push(remainder);
        if next_start == words.len() {
            break;
        }
        core::mem::swap(&mut words, &mut scratch);
        start = next_start;
    }

    let ms_digits = count_base58_digits(chunks[chunks.len() - 1]);
    let out_len = zcount + ms_digits + (chunks.len() - 1) * CHUNK_DIGITS;
    let mut out = vec![alphabet.zero_char(); out_len];

    let mut pos = out.len();
    for &chunk in &chunks[..chunks.len() - 1] {
        let mut chunk = chunk;
        for _ in 0..CHUNK_DIGITS {
            pos -= 1;
            out[pos] = alphabet.encode_digit(chunk % 58);
            chunk /= 58;
        }
    }

    let mut chunk = chunks[chunks.len() - 1];
    while chunk > 0 {
        pos -= 1;
        out[pos] = alphabet.encode_digit(chunk % 58);
        chunk /= 58;
    }

    String::from_utf8(out).expect("alphabet bytes are ascii")
}

pub fn decode(input: &[u8], alphabet: &Alphabet) -> Result<Vec<u8>, DecodeError> {
    if input.is_empty() {
        return Err(DecodeError::ZeroLengthString);
    }

    let payload_bytes = input;
    let zero = alphabet.zero_char();
    let b58sz = payload_bytes.len();

    let zcount = payload_bytes.iter().take_while(|&&b| b == zero).count();
    if zcount == b58sz {
        return Ok(vec![0u8; zcount]);
    }

    let payload = &payload_bytes[zcount..];
    let byte_estimate = payload.len() * 406 / 555 + 1;
    let word_cap = byte_estimate.div_ceil(8);
    let mut words: Vec<u64> = Vec::with_capacity(word_cap);

    let first_chunk_digits = match payload.len() % CHUNK_DIGITS {
        0 => CHUNK_DIGITS,
        n => n,
    };

    let mut offset = 0usize;
    while offset < payload.len() {
        let chunk_digits = if offset == 0 {
            first_chunk_digits
        } else {
            CHUNK_DIGITS
        };

        let mut chunk = 0u64;
        for i in 0..chunk_digits {
            let ch = payload[offset + i];
            if ch > 127 {
                return Err(DecodeError::HighBit(ch));
            }
            let val = alphabet.decode_byte(ch);
            if val == -1 {
                return Err(DecodeError::InvalidDigit(ch));
            }
            chunk = chunk * 58 + val as u64;
        }
        mul_add_words_le(&mut words, CHUNK_POWERS[chunk_digits], chunk);
        offset += chunk_digits;
    }

    Ok(unpack_words_le(&words, zcount))
}

fn load_base256_words(src: &[u8], word_count: usize) -> Vec<u64> {
    let mut dst = vec![0u64; word_count];
    let first = match src.len() % 8 {
        0 => 8,
        n => n,
    };

    let mut word = 0u64;
    for &b in &src[..first] {
        word = (word << 8) | b as u64;
    }
    dst[0] = word;

    let mut offset = first;
    let mut i = 1;
    while offset < src.len() {
        let bytes: [u8; 8] = src[offset..offset + 8].try_into().expect("8 bytes");
        dst[i] = u64::from_be_bytes(bytes);
        offset += 8;
        i += 1;
    }

    dst
}

#[inline]
fn div_wide(hi: u64, lo: u64, divisor: u64) -> (u64, u64) {
    let n = ((hi as u128) << 64) | lo as u128;
    let d = divisor as u128;
    ((n / d) as u64, (n % d) as u64)
}

fn count_base58_digits(mut v: u64) -> usize {
    let mut digits = 0;
    while v > 0 {
        digits += 1;
        v /= 58;
    }
    digits.max(1)
}

fn mul_add_words_le(words: &mut Vec<u64>, mul: u64, add: u64) {
    if words.is_empty() {
        if add != 0 {
            words.push(add);
        }
        return;
    }

    let mut carry = add;
    for word in words.iter_mut() {
        let product = *word as u128 * mul as u128;
        let hi = (product >> 64) as u64;
        let lo = product as u64;
        let (sum, overflow) = lo.overflowing_add(carry);
        *word = sum;
        carry = hi + overflow as u64;
    }
    if carry != 0 {
        words.push(carry);
    }
}

fn unpack_words_le(words: &[u64], zcount: usize) -> Vec<u8> {
    let mut high = words.len();
    while high > 0 && words[high - 1] == 0 {
        high -= 1;
    }
    if high == 0 {
        return vec![0u8; zcount];
    }
    let high = high - 1;

    let ms_bytes = (64 - words[high].leading_zeros() as usize).div_ceil(8);
    let mut out = vec![0u8; zcount + ms_bytes + 8 * high];
    let mut pos = zcount;

    let top = words[high].to_be_bytes();
    out[pos..pos + ms_bytes].copy_from_slice(&top[8 - ms_bytes..]);
    pos += ms_bytes;

    for i in (0..high).rev() {
        out[pos..pos + 8].copy_from_slice(&words[i].to_be_bytes());
        pos += 8;
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_u128(words: &[u64]) -> u128 {
        let mut n = 0u128;
        for &w in words.iter().rev() {
            n = (n << 64) | w as u128;
        }
        n
    }

    #[test]
    fn mul_add_matches_u128_reference() {
        let word_sets: [&[u64]; 6] = [
            &[],
            &[0],
            &[1],
            &[57],
            &[u32::MAX as u64],
            &[123_456_789, 3],
        ];
        let addends = [0u64, 1, 57, 58, 123_456_789, u32::MAX as u64];

        for &mul in CHUNK_POWERS.iter().skip(1) {
            for words in word_sets {
                for &add in &addends {
                    let want = to_u128(words) * mul as u128 + add as u128;
                    let mut got = words.to_vec();
                    mul_add_words_le(&mut got, mul, add);
                    assert_eq!(to_u128(&got), want, "words={words:?} mul={mul} add={add}");
                }
            }
        }
    }
}
