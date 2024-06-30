use std::fmt::Write;

use super::{BinaryOp, Node, NodeRef, Token, UnuaryOp, Value};

pub fn serialize<T: FnMut(Token)>(node: NodeRef, f: &mut T) {
    match &*node {
        Node::Value(val) => match val {
            Value::Str(val) => f(Token::String(val.clone())),
            Value::Int(val) => {
                if val < &(0u32.into()) {
                    f(Token::UnaryMinus);
                    f(Token::Integer((-val).try_into().unwrap()));
                } else {
                    f(Token::Integer(val.try_into().unwrap()));
                }
            }
            Value::Bool(val) => f(if *val { Token::True } else { Token::False }),
        },
        Node::Lambda { var, body } => {
            f(Token::Lambda(var.id()));
            serialize(body.clone(), f);
        }
        Node::Variable(var_id) => f(Token::Variable(var_id.id())),
        Node::Apply { f: func, value } => {
            f(Token::Apply);
            serialize(func.clone(), f);
            serialize(value.clone(), f);
        }
        Node::BinaryOp { op, left, right } => {
            f(match *op {
                BinaryOp::IntAdd => Token::Add,
                BinaryOp::IntSub => Token::Subtract,
                BinaryOp::IntMul => Token::Multiply,
                BinaryOp::IntDiv => Token::Divide,
                BinaryOp::IntMod => Token::Modulo,
                BinaryOp::IntLt => Token::LessThan,
                BinaryOp::IntGt => Token::GreaterThan,
                BinaryOp::BoolOr => Token::Or,
                BinaryOp::BoolAnd => Token::And,
                BinaryOp::StrConcat => Token::StringConcat,
                BinaryOp::StrTake => Token::Take,
                BinaryOp::StrDrop => Token::Drop,
                BinaryOp::Eq => Token::Equal,
            });
            serialize(left.clone(), f);
            serialize(right.clone(), f);
        }
        Node::UnuaryOp { op, body } => {
            f(match *op {
                UnuaryOp::IntNeg => Token::UnaryMinus,
                UnuaryOp::BoolNot => Token::UnaryNot,
                UnuaryOp::StrToInt => Token::StringToInt,
                UnuaryOp::IntToStr => Token::IntToString,
            });
            serialize(body.clone(), f);
        }
        Node::If {
            cond,
            then_do,
            else_do,
        } => {
            f(Token::If);
            serialize(cond.clone(), f);
            serialize(then_do.clone(), f);
            serialize(else_do.clone(), f);
        }
    }
}

pub fn serialize_str(node: NodeRef) -> String {
    let mut res = String::new();
    serialize(node, &mut |token| {
        let _ = write!(&mut res, "{} ", token);
    });
    res.pop();
    res
}

#[cfg(test)]
mod tests {
    use logos::Logos;

    use super::*;

    #[test]
    fn loopback() {
        let reference: &str = r#"? B= B$ B$ B$ B$ L$ L$ L$ L# v$ I" I# I$ I% I$ ? B= B$ L$ v$ I+ I+ ? B= BD I$ S4%34 S4 ? B= BT I$ S4%34 S4%3 ? B= B. S4% S34 S4%34 ? U! B& T F ? B& T T ? U! B| F F ? B| F T ? B< U- I$ U- I# ? B> I$ I# ? B= U- I" B% U- I$ I# ? B= I" B% I( I$ ? B= U- I" B/ U- I$ I# ? B= I# B/ I( I$ ? B= I' B* I# I$ ? B= I$ B+ I" I# ? B= U$ I4%34 S4%34 ? B= U# S4%34 I4%34 ? U! F ? B= U- I$ B- I# I& ? B= I$ B- I& I# ? B= S4%34 S4%34 ? B= F F ? B= I$ I$ ? T B. B. SM%,&k#(%#+}IEj}3%.$}z3/,6%},!.'5!'%y4%34} U$ B+ I# B* I$> I1~s:U@ Sz}4/}#,!)-}0/).43}&/2})4 S)&})3}./4}#/22%#4 S").!29}q})3}./4}#/22%#4 S").!29}q})3}./4}#/22%#4 S").!29}q})3}./4}#/22%#4 S").!29}k})3}./4}#/22%#4 S5.!29}k})3}./4}#/22%#4 S5.!29}_})3}./4}#/22%#4 S5.!29}a})3}./4}#/22%#4 S5.!29}b})3}./4}#/22%#4 S").!29}i})3}./4}#/22%#4 S").!29}h})3}./4}#/22%#4 S").!29}m})3}./4}#/22%#4 S").!29}m})3}./4}#/22%#4 S").!29}c})3}./4}#/22%#4 S").!29}c})3}./4}#/22%#4 S").!29}r})3}./4}#/22%#4 S").!29}p})3}./4}#/22%#4 S").!29}{})3}./4}#/22%#4 S").!29}{})3}./4}#/22%#4 S").!29}d})3}./4}#/22%#4 S").!29}d})3}./4}#/22%#4 S").!29}l})3}./4}#/22%#4 S").!29}N})3}./4}#/22%#4 S").!29}>})3}./4}#/22%#4 S!00,)#!4)/.})3}./4}#/22%#4 S!00,)#!4)/.})3}./4}#/22%#4"#;
        let mut lexer = crate::icfp::Token::lexer(&reference);
        let ast = crate::icfp::parse(&mut lexer).unwrap();
        assert_eq!(&serialize_str(ast), reference);
    }
}
