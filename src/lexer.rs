use logos::{Lexer, Logos};

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r" ")]
pub enum Token {
    #[token("T")]
    True,
    #[token("F")]
    False,
    #[regex("I[\u{0021}-\u{007E}]+", integer)]
    Integer(u64),
    #[regex("S[\u{0021}-\u{007E}]+", string)]
    String(String),

    #[token("U-")]
    UnaryMinus,
    #[token("U!")]
    UnaryNot,
    #[token("U#")]
    StringToInt,
    #[token("U$")]
    IntToString,

    #[token("B+")]
    Add,
    #[token("B-")]
    Subtract,
    #[token("B*")]
    Multiply,
    #[token("B/")]
    Divide,
    #[token("B%")]
    Modulo,
    #[token("B<")]
    LessThan,
    #[token("B>")]
    GreaterThan,
    #[token("B=")]
    Equal,
    #[token("B|")]
    Or,
    #[token("B&")]
    And,
    #[token("B.")]
    StringConcat,
    #[token("BT")]
    Take,
    #[token("BD")]
    Drop,
    #[token("B$")]
    Apply,

    #[token("?")]
    If,

    #[regex("L[\u{0021}-\u{007E}]+", integer)]
    Lambda(u64),
    #[regex("v[\u{0021}-\u{007E}]+", integer)]
    Variable(u64),
}

fn integer(lex: &mut Lexer<Token>) -> Option<u64> {
    from_base94(&lex.slice()[1..])
}

pub fn from_base94(s: &str) -> Option<u64> {
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

pub fn to_base94(s: u64) -> String {
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

fn string(lex: &mut Lexer<Token>) -> String {
    let slice = &lex.slice()[1..];
    map_string(slice)
}

const MAPPING: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!\"#$%&'()*+,-./:;<=>?@[\\]^_`|~ \n";

pub fn map_string(s: &str) -> String {
    let mut bytes = vec![];
    for c in s.chars() {
        bytes.push(MAPPING.as_bytes()[c as usize - 33]);
    }
    unsafe { String::from_utf8_unchecked(bytes) }
}

pub fn unmap_string(s: &str) -> String {
    let mut bytes = vec![];
    for c in s.chars() {
        let idx = MAPPING.find(c).unwrap();
        bytes.push((idx + 33) as u8);
    }
    unsafe { String::from_utf8_unchecked(bytes) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bool_true() {
        let mut lex = Token::lexer("T");
        assert_eq!(lex.next().unwrap().unwrap(), Token::True);
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn bool_false() {
        let mut lex = Token::lexer("F");
        assert_eq!(lex.next().unwrap().unwrap(), Token::False);
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn integer() {
        let mut lex = Token::lexer("I/6");
        assert_eq!(lex.next().unwrap().unwrap(), Token::Integer(1337));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn string() {
        let mut lex = Token::lexer("SB%,,/}Q/2,$_");
        assert_eq!(
            lex.next().unwrap().unwrap(),
            Token::String("Hello World!".to_string())
        );
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn eval_example() {
        let mut lex = Token::lexer("B$ L# B$ L\" B+ v\" v\" B* I$ I# v8");
        assert_eq!(lex.next().unwrap().unwrap(), Token::Apply);
        assert_eq!(lex.next().unwrap().unwrap(), Token::Lambda(2));
        assert_eq!(lex.next().unwrap().unwrap(), Token::Apply);
        assert_eq!(lex.next().unwrap().unwrap(), Token::Lambda(1));
        assert_eq!(lex.next().unwrap().unwrap(), Token::Add);
        assert_eq!(lex.next().unwrap().unwrap(), Token::Variable(1));
        assert_eq!(lex.next().unwrap().unwrap(), Token::Variable(1));
        assert_eq!(lex.next().unwrap().unwrap(), Token::Multiply);
        assert_eq!(lex.next().unwrap().unwrap(), Token::Integer(3));
        assert_eq!(lex.next().unwrap().unwrap(), Token::Integer(2));
        assert_eq!(lex.next().unwrap().unwrap(), Token::Variable(23));
    }
}
