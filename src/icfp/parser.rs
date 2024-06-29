use std::rc::Rc;

use super::{
    Token, {BinaryOp, Node, NodeRef, UnuaryOp, Value, VarId},
};
use logos::Lexer;

#[derive(Debug, Clone)]
pub enum ParsingError {
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
        Token::False => Rc::new(Node::Value(Value::Bool(false))),
        Token::Integer(value) => Rc::new(Node::Value(Value::Int(value as i64))),
        Token::String(value) => Rc::new(Node::Value(Value::Str(value))),

        // unuary
        Token::UnaryMinus => parse_unuary(lexer, UnuaryOp::IntNeg)?,
        Token::UnaryNot => parse_unuary(lexer, UnuaryOp::BoolNot)?,
        Token::StringToInt => parse_unuary(lexer, UnuaryOp::StrToInt)?,
        Token::IntToString => parse_unuary(lexer, UnuaryOp::IntToStr)?,

        // binary
        Token::Add => parse_bin(lexer, BinaryOp::IntAdd)?,
        Token::Subtract => parse_bin(lexer, BinaryOp::IntSub)?,
        Token::Multiply => parse_bin(lexer, BinaryOp::IntMul)?,
        Token::Divide => parse_bin(lexer, BinaryOp::IntDiv)?,
        Token::Modulo => parse_bin(lexer, BinaryOp::IntMod)?,
        Token::LessThan => parse_bin(lexer, BinaryOp::IntLt)?,
        Token::GreaterThan => parse_bin(lexer, BinaryOp::IntGt)?,
        Token::Equal => parse_bin(lexer, BinaryOp::Eq)?,
        Token::Or => parse_bin(lexer, BinaryOp::BoolOr)?,
        Token::And => parse_bin(lexer, BinaryOp::BoolAnd)?,
        Token::StringConcat => parse_bin(lexer, BinaryOp::StrConcat)?,
        Token::Take => parse_bin(lexer, BinaryOp::StrTake)?,
        Token::Drop => parse_bin(lexer, BinaryOp::StrDrop)?,

        // flow control / scoping
        Token::If => Rc::new(Node::If {
            cond: parse(lexer)?,
            then_do: parse(lexer)?,
            else_do: parse(lexer)?,
        }),
        Token::Lambda(id) => Rc::new(Node::Lambda {
            var: VarId::new(id),
            body: parse(lexer)?,
        }),
        Token::Variable(id) => Rc::new(Node::Variable(VarId::new(id))),
        Token::Apply => Rc::new(Node::Apply {
            f: parse(lexer)?,
            value: parse(lexer)?,
        }),
    })
}
