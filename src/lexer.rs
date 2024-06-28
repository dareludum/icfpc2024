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
    let slice = &lex.slice()[1..];

    let mut result = 0u64;
    let mut power = 1u64;
    const BASE: u64 = 94;
    for c in slice.chars().rev() {
        let digit = c as u64 - 33; // Subtract 33 to convert from ASCII to base-94
        if digit >= BASE {
            return None; // Invalid character
        }
        result += digit * power;
        power *= BASE;
    }

    Some(result)
}

fn string(lex: &mut Lexer<Token>) -> Option<String> {
    const MAPPING: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!\"#$%&'()*+,-./:;<=>?@[\\]^_`|~ \n";
    let slice = &lex.slice()[1..];
    let mut bytes = vec![];
    for c in slice.chars() {
        bytes.push(MAPPING.as_bytes()[c as usize - 33]);
    }
    Some(unsafe { String::from_utf8_unchecked(bytes) })
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

    #[test]
    fn get_index_response() {
        let mut lex = Token::lexer("SJ!23%}%22/2n}O.%80%#4%$}#(!2!#4%2}eee}!4}).$%8}U");
        assert_eq!(lex.next().unwrap().unwrap(), Token::Integer(1337));
        assert_eq!(lex.next(), None);
    }
}
