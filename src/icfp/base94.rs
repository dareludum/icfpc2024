const ALPHABET: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!\"#$%&'()*+,-./:;<=>?@[\\]^_`|~ \n";

pub fn base94_to_int(s: &str) -> Option<u64> {
    let mut result = 0u64;
    let mut power = 1u64;
    const BASE: u64 = 94;
    for c in s.chars().rev() {
        let digit = c as u64 - 33; // Subtract 33 to convert from ASCII to base-94
        if digit >= BASE {
            return None; // Invalid character
        }
        result += digit * power;
        power *= BASE;
    }
    Some(result)
}

pub fn int_to_base94(s: u64) -> String {
    let mut result = String::new();
    let mut slice = s;
    const BASE: u64 = 94;
    while slice > 0 {
        let digit = slice % BASE;
        result.push((digit + 33) as u8 as char); // Add 33 to convert from base-94 to ASCII
        slice /= BASE;
    }
    result.chars().rev().collect()
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
