use base58::{
    decode, decode_alphabet, encode, encode_alphabet, trivial_decode_alphabet,
    trivial_encode_alphabet, Alphabet, DecodeError, BTC_ALPHABET, FLICKR_ALPHABET,
};

const BTC_DIGITS: &str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

fn from_hex(s: &str) -> Vec<u8> {
    assert!(s.len() % 2 == 0, "odd hex length");
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("valid hex"))
        .collect()
}

fn reverse(s: &str) -> String {
    s.chars().rev().collect()
}

#[test]
fn known_vectors() {
    let cases = [
        ("", ""),
        ("61", "2g"),
        ("626262", "a3gV"),
        ("636363", "aPEr"),
        (
            "73696d706c792061206c6f6e6720737472696e67",
            "2cFupjhnEsSn59qHXstmK2ffpLv2",
        ),
        ("516b6fcd0f", "ABnLTmg"),
        ("bf4f89001e670274dd", "3SEo3LWLoPntC"),
        ("572e4794", "3EFU7m"),
        ("ecac89cad93923c02321", "EJDM8drfXA6uyA"),
        ("10c8511e", "Rt5zm"),
        ("00000000000000000000", "1111111111"),
    ];

    for (hex, enc) in cases {
        let data = from_hex(hex);
        assert_eq!(encode(&data), enc, "encode {hex}");
        assert_eq!(
            trivial_encode_alphabet(&data, &BTC_ALPHABET),
            enc,
            "trivial encode {hex}"
        );

        if enc.is_empty() {
            continue;
        }

        assert_eq!(decode(enc).unwrap(), data, "decode {enc}");
        assert_eq!(
            trivial_decode_alphabet(enc, &BTC_ALPHABET).unwrap(),
            data,
            "trivial decode {enc}"
        );
    }
}

#[test]
fn btc_addresses_round_trip() {
    let addresses = [
        "1QCaxc8hutpdZ62iKZsn1TCG3nh7uPZojq",
        "1DhRmSGnhPjUaVPAj48zgPV9e2oRhAQFUb",
        "17LN2oPYRYsXS9TdYdXCCDvF2FegshLDU2",
        "14h2bDLZSuvRFhUL45VjPHJcW667mmRAAn",
    ];
    for addr in addresses {
        let decoded = decode(addr).unwrap();
        assert_eq!(encode(&decoded), addr);
    }
}

#[test]
fn leading_zeros_preserved_across_chunk_boundaries() {
    let custom = Alphabet::new(&reverse(BTC_DIGITS)).unwrap();
    let alphabets: [&Alphabet; 3] = [&BTC_ALPHABET, &FLICKR_ALPHABET, &custom];
    let tails: [Vec<u8>; 5] = [
        vec![],
        vec![1],
        vec![1, 2, 3, 4, 5],
        from_hex("0102030405060708090a0b0c0d0e0f10"),
        from_hex("ffffffffffffffffffffffffffffffff"),
    ];

    for alphabet in alphabets {
        for zero_count in [1, 2, 7, 8, 9, 10, 11, 32, 64] {
            for tail in &tails {
                let mut payload = vec![0u8; zero_count];
                payload.extend_from_slice(tail);

                let encoded = encode_alphabet(&payload, alphabet);
                let decoded = decode_alphabet(&encoded, alphabet).unwrap();
                assert_eq!(decoded, payload, "zeros={zero_count}");
            }
        }
    }
}

#[test]
fn fast_matches_trivial_on_boundary_payloads() {
    let custom = Alphabet::new(&format!("{}{}", &BTC_DIGITS[17..], &BTC_DIGITS[..17])).unwrap();
    let alphabets: [&Alphabet; 3] = [&BTC_ALPHABET, &FLICKR_ALPHABET, &custom];
    let lengths = [
        1, 2, 7, 8, 9, 10, 11, 15, 16, 17, 31, 32, 33, 63, 64, 65, 127, 128, 129, 255,
    ];
    let zero_counts = [0, 1, 2, 7, 8, 9];

    for alphabet in alphabets {
        for length in lengths {
            for zero_count in zero_counts {
                let mut payload = vec![0u8; zero_count];
                payload
                    .extend((0..length).map(|i: usize| ((i * 37 + length * 17) % 251 + 1) as u8));

                let fast = encode_alphabet(&payload, alphabet);
                let trivial = trivial_encode_alphabet(&payload, alphabet);
                assert_eq!(fast, trivial, "len={length} zeros={zero_count}");

                let decoded = decode_alphabet(&fast, alphabet).unwrap();
                assert_eq!(decoded, payload, "len={length} zeros={zero_count}");
                let trivial_decoded = trivial_decode_alphabet(&trivial, alphabet).unwrap();
                assert_eq!(trivial_decoded, payload, "len={length} zeros={zero_count}");
            }
        }
    }
}

#[test]
fn decode_rejects_malformed_inputs() {
    let cases: [(&[u8], DecodeError); 8] = [
        (b"", DecodeError::ZeroLengthString),
        (b"0", DecodeError::InvalidDigit(b'0')),
        (b"O", DecodeError::InvalidDigit(b'O')),
        (b"I", DecodeError::InvalidDigit(b'I')),
        (b"l", DecodeError::InvalidDigit(b'l')),
        (b"12 3", DecodeError::InvalidDigit(b' ')),
        (&[0x80], DecodeError::HighBit(0x80)),
        (b"11\xff", DecodeError::HighBit(0xff)),
    ];

    for (input, want) in cases {
        assert_eq!(decode(input), Err(want.clone()), "fast decode {input:?}");
        assert_eq!(
            trivial_decode_alphabet(input, &BTC_ALPHABET),
            Err(want),
            "trivial decode {input:?}"
        );
    }
}

#[test]
fn error_messages_contain_expected_substrings() {
    assert!(decode("")
        .unwrap_err()
        .to_string()
        .contains("zero length string"));
    assert!(decode("0")
        .unwrap_err()
        .to_string()
        .contains("invalid base58 digit"));
    assert!(decode([0x80])
        .unwrap_err()
        .to_string()
        .contains("high-bit set on invalid digit"));
}

#[test]
fn alphabet_validation() {
    assert!(Alphabet::new(&BTC_DIGITS[1..]).is_err());
    assert!(Alphabet::new(&format!("0{BTC_DIGITS}")).is_err());
    assert!(Alphabet::new(&format!("\u{00ff}{}", &BTC_DIGITS[1..])).is_err());
    assert!(Alphabet::new(&format!("z{}", &BTC_DIGITS[1..])).is_err());
    assert!(Alphabet::new(BTC_DIGITS).is_ok());
}
