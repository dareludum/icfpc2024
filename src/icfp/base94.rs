use num::{BigInt, BigUint};

const ALPHABET: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!\"#$%&'()*+,-./:;<=>?@[\\]^_`|~ \n";

pub type Base94UInt = BigUint;
pub type Base94Int = BigInt;

pub fn base94_to_int(s: &str) -> Option<BigUint> {
    let bytes = s.as_bytes().iter().map(|b| b - 33).collect::<Vec<_>>();
    BigUint::from_radix_be(&bytes, 94)
}

pub fn int_to_base94(s: &BigUint) -> String {
    s.to_radix_be(94)
        .iter()
        .map(|&b| b + 33)
        .map(|b| b as char)
        .collect()
}

pub fn base94_to_str(s: &str) -> String {
    let mut bytes = vec![];
    for c in s.chars() {
        bytes.push(ALPHABET.as_bytes()[c as usize - 33]);
    }
    unsafe { String::from_utf8_unchecked(bytes) }
}

pub fn str_to_base94(s: &str) -> String {
    let mut bytes = vec![];
    for c in s.chars() {
        let idx = ALPHABET.find(c).unwrap();
        bytes.push((idx + 33) as u8);
    }
    unsafe { String::from_utf8_unchecked(bytes) }
}
