use crate::alphabet::Alphabet;
use crate::error::DecodeError;

pub fn encode(bin: &[u8], alphabet: &Alphabet) -> String {
    let zcount = bin.iter().take_while(|&&b| b == 0).count();

    let mut digits: Vec<u8> = Vec::new();
    for &b in bin {
        let mut carry = b as u32;
        for digit in digits.iter_mut() {
            carry += (*digit as u32) << 8;
            *digit = (carry % 58) as u8;
            carry /= 58;
        }
        while carry > 0 {
            digits.push((carry % 58) as u8);
            carry /= 58;
        }
    }

    let mut out = Vec::with_capacity(zcount + digits.len());
    out.resize(zcount, alphabet.zero_char());
    for &d in digits.iter().rev() {
        out.push(alphabet.encode_digit(d as u64));
    }

    String::from_utf8(out).expect("alphabet bytes are ascii")
}

pub fn decode(input: &[u8], alphabet: &Alphabet) -> Result<Vec<u8>, DecodeError> {
    if input.is_empty() {
        return Err(DecodeError::ZeroLengthString);
    }

    let zero = alphabet.zero_char();
    let zcount = input.iter().take_while(|&&b| b == zero).count();
    if zcount == input.len() {
        return Ok(vec![0u8; zcount]);
    }

    let mut number: Vec<u8> = Vec::new();
    for &ch in &input[zcount..] {
        if ch > 127 {
            return Err(DecodeError::HighBit(ch));
        }
        let val = alphabet.decode_byte(ch);
        if val == -1 {
            return Err(DecodeError::InvalidDigit(ch));
        }

        let mut carry = val as u32;
        for byte in number.iter_mut().rev() {
            carry += *byte as u32 * 58;
            *byte = (carry & 0xff) as u8;
            carry >>= 8;
        }
        while carry > 0 {
            number.insert(0, (carry & 0xff) as u8);
            carry >>= 8;
        }
    }

    let mut out = vec![0u8; zcount];
    out.extend_from_slice(&number);
    Ok(out)
}
