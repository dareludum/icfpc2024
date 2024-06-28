use std::rc::Rc;

use crate::{
    ast::{BinaryOp, Node, NodeRef, UnuaryOp, Value, VarId},
    lexer::Token,
};
use logos::{Lexer, Logos};

enum ParsingError {
    EmptyTokenStream,
    LexerError,
}

fn parse_unuary(lexer: &mut Lexer<Token>, op: UnuaryOp) -> Result<NodeRef, ParsingError> {
    Ok(Rc::new(Node::UnuaryOp {
        op,
        body: parse(lexer)?,
    }))
}

fn parse_bin(lexer: &mut Lexer<Token>, op: BinaryOp) -> Result<NodeRef, ParsingError> {
    Ok(Rc::new(Node::BinaryOp {
        op,
        left: parse(lexer)?,
        right: parse(lexer)?,
    }))
}

pub fn parse(lexer: &mut Lexer<Token>) -> Result<NodeRef, ParsingError> {
    let Some(token) = lexer.next() else {
        return Err(ParsingError::EmptyTokenStream);
    };
    let token = token.map_err(|_| ParsingError::LexerError)?;
    Ok(match token {
        // litterals
        Token::True => Rc::new(Node::Value(Value::Bool(true))),
        Token::False => Rc::new(Node::Value(Value::Bool(true))),
        Token::Integer(value) => Rc::new(Node::Value(Value::Int(value as i64))),
        Token::String(value) => Rc::new(Node::Value(Value::Str(value))),

        // unuary
        Token::UnaryMinus => parse_unuary(lexer, UnuaryOp::IntNeg)?,
        Token::UnaryNot => parse_unuary(lexer, UnuaryOp::BoolNot)?,
        Token::StringToInt => parse_unuary(lexer, UnuaryOp::StrToInt)?,
        Token::IntToString => parse_unuary(lexer, UnuaryOp::IntToStr)?,

        // binary
        Token::Add => parse_bin(lexer, BinaryOp::IntAdd)?,
        Token::Subtract => parse(lexer)?,
        Token::Multiply => parse(lexer)?,
        Token::Divide => parse(lexer)?,
        Token::Modulo => parse(lexer)?,
        Token::LessThan => parse(lexer)?,
        Token::GreaterThan => parse(lexer)?,
        Token::Equal => parse(lexer)?,
        Token::Or => parse(lexer)?,
        Token::And => parse(lexer)?,
        Token::StringConcat => parse(lexer)?,
        Token::Take => parse(lexer)?,
        Token::Drop => parse(lexer)?,
        Token::Apply => parse(lexer)?,

        // flow control / scoping
        Token::If => parse(lexer)?,
        Token::Lambda(id) => Rc::new(Node::Lambda {
            var: VarId::new(id),
            body: parse(lexer)?,
        }),
        Token::Variable(id) => Rc::new(Node::Variable(VarId::new(id))),
    })
}
