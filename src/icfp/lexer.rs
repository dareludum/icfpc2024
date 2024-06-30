use std::fmt::Write;

use logos::{Lexer, Logos};

use super::{
    base94::{self, Base94UInt},
    base94_to_int, base94_to_str,
};

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r" ")]
pub enum Token {
    #[token("T")]
    True,
    #[token("F")]
    False,
    #[regex("I[\u{0021}-\u{007E}]+", u94_integer)]
    Integer(Base94UInt),
    #[regex("S[\u{0021}-\u{007E}]*", string)]
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

    #[regex("L[\u{0021}-\u{007E}]+", u64_integer)]
    Lambda(u64),
    #[regex("v[\u{0021}-\u{007E}]+", u64_integer)]
    Variable(u64),
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::True => f.write_char('T'),
            Token::False => f.write_char('F'),
            Token::UnaryMinus => f.write_str("U-"),
            Token::UnaryNot => f.write_str("U!"),
            Token::StringToInt => f.write_str("U#"),
            Token::IntToString => f.write_str("U$"),
            Token::Add => f.write_str("B+"),
            Token::Subtract => f.write_str("B-"),
            Token::Multiply => f.write_str("B*"),
            Token::Divide => f.write_str("B/"),
            Token::Modulo => f.write_str("B%"),
            Token::LessThan => f.write_str("B<"),
            Token::GreaterThan => f.write_str("B>"),
            Token::Equal => f.write_str("B="),
            Token::Or => f.write_str("B|"),
            Token::And => f.write_str("B&"),
            Token::StringConcat => f.write_str("B."),
            Token::Take => f.write_str("BT"),
            Token::Drop => f.write_str("BD"),
            Token::Apply => f.write_str("B$"),
            Token::If => f.write_char('?'),
            Token::Lambda(val) => {
                f.write_char('L')?;
                f.write_str(&base94::int_to_base94(&(*val).into()))
            }
            Token::Variable(val) => {
                f.write_char('v')?;
                f.write_str(&base94::int_to_base94(&(*val).into()))
            }
            Token::Integer(val) => {
                f.write_char('I')?;
                f.write_str(&base94::int_to_base94(val))
            }
            Token::String(val) => {
                f.write_char('S')?;
                f.write_str(&base94::str_to_base94(val))
            }
        }
    }
}

fn u94_integer(lex: &mut Lexer<Token>) -> Option<Base94UInt> {
    base94_to_int(&lex.slice()[1..])
}

fn u64_integer(lex: &mut Lexer<Token>) -> u64 {
    u94_integer(lex).unwrap().iter_u64_digits().next().unwrap()
}

fn string(lex: &mut Lexer<Token>) -> String {
    let slice = &lex.slice()[1..];
    base94_to_str(slice)
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
        assert_eq!(lex.next().unwrap().unwrap(), Token::Integer(1337u32.into()));
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
        assert_eq!(lex.next().unwrap().unwrap(), Token::Lambda(2u32.into()));
        assert_eq!(lex.next().unwrap().unwrap(), Token::Apply);
        assert_eq!(lex.next().unwrap().unwrap(), Token::Lambda(1u32.into()));
        assert_eq!(lex.next().unwrap().unwrap(), Token::Add);
        assert_eq!(lex.next().unwrap().unwrap(), Token::Variable(1u32.into()));
        assert_eq!(lex.next().unwrap().unwrap(), Token::Variable(1u32.into()));
        assert_eq!(lex.next().unwrap().unwrap(), Token::Multiply);
        assert_eq!(lex.next().unwrap().unwrap(), Token::Integer(3u32.into()));
        assert_eq!(lex.next().unwrap().unwrap(), Token::Integer(2u32.into()));
        assert_eq!(lex.next().unwrap().unwrap(), Token::Variable(23u32.into()));
    }

    #[test]
    fn lambdaman10() {
        let lex = Token::lexer("B. SF B$ B$ L\" B$ L# B$ v\" B$ v# v# L# B$ v\" B$ v# v# L\" L# ? B= v# I;Y S B. ? B= B% v# IS I! S~ S B. ? B= B% v# I, I! Sa Sl B$ v\" B+ v# I\" I\"");
        lex.collect::<Result<Vec<_>, _>>()
            .expect("Failed to lex the program");
    }
}
