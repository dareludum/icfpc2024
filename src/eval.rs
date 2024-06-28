use std::rc::Rc;

use crate::ast::{BinaryOp, Node, UnuaryOp, Value, VarId};

pub fn evaluate(tree: Rc<Node>) -> Value {
    match evaluate_node(tree).as_ref() {
        Node::Value(v) => v.clone(),
        _ => panic!("Expected value"),
    }
}

fn evaluate_node(tree: Rc<Node>) -> Rc<Node> {
    match tree.as_ref() {
        Node::Value(_) => tree,
        Node::Lambda { .. } => tree,
        Node::Variable(_) => panic!("Unsubstituted variable"),
        Node::Apply { f, value } => {
            let node = evaluate_node(f.clone());
            match node.as_ref() {
                Node::Lambda { var, body } => {
                    evaluate_node(substitute(body.clone(), *var, value.clone()))
                }
                _ => panic!("Invalid application"),
            }
        }
        Node::BinaryOp { op, left, right } => {
            let left = evaluate_node(left.clone());
            let right = evaluate_node(right.clone());
            match (left.as_ref(), right.as_ref()) {
                (Node::Value(l), Node::Value(r)) => match op {
                    BinaryOp::IntAdd => Rc::new(Node::Value(Value::Int(l.as_int() + r.as_int()))),
                    BinaryOp::IntSub => Rc::new(Node::Value(Value::Int(l.as_int() - r.as_int()))),
                    BinaryOp::IntMul => Rc::new(Node::Value(Value::Int(l.as_int() * r.as_int()))),
                    BinaryOp::IntDiv => Rc::new(Node::Value(Value::Int(l.as_int() / r.as_int()))),
                    BinaryOp::IntMod => Rc::new(Node::Value(Value::Int(l.as_int() % r.as_int()))),
                    BinaryOp::IntLt => Rc::new(Node::Value(Value::Bool(l.as_int() < r.as_int()))),
                    BinaryOp::IntGt => Rc::new(Node::Value(Value::Bool(l.as_int() > r.as_int()))),
                    BinaryOp::BoolOr => {
                        Rc::new(Node::Value(Value::Bool(l.as_bool() || r.as_bool())))
                    }
                    BinaryOp::BoolAnd => {
                        Rc::new(Node::Value(Value::Bool(l.as_bool() && r.as_bool())))
                    }
                    BinaryOp::StrConcat => Rc::new(Node::Value(Value::Str(format!(
                        "{}{}",
                        l.as_str(),
                        r.as_str()
                    )))),
                    BinaryOp::StrTake => Rc::new(Node::Value(Value::Str(
                        r.as_str().chars().take(l.as_int() as usize).collect(),
                    ))),
                    BinaryOp::StrDrop => Rc::new(Node::Value(Value::Str(
                        r.as_str().chars().skip(l.as_int() as usize).collect(),
                    ))),
                    BinaryOp::Eq => Rc::new(Node::Value(Value::Bool(l == r))),
                },
                _ => panic!("Invalid binary operation"),
            }
        }
        Node::UnuaryOp { op, body } => {
            let body = evaluate_node(body.clone());
            match body.as_ref() {
                Node::Value(v) => match op {
                    UnuaryOp::IntNeg => Rc::new(Node::Value(Value::Int(-v.as_int()))),
                    UnuaryOp::BoolNot => Rc::new(Node::Value(Value::Bool(!v.as_bool()))),
                    UnuaryOp::StrToInt => {
                        Rc::new(Node::Value(Value::Int(v.as_str().parse().unwrap())))
                    }
                    UnuaryOp::IntToStr => Rc::new(Node::Value(Value::Str(v.as_int().to_string()))),
                },
                _ => panic!("Invalid unary operation"),
            }
        }
        Node::If {
            cond,
            then_do,
            else_do,
        } => {
            let cond = evaluate_node(cond.clone());
            match cond.as_ref() {
                Node::Value(Value::Bool(b)) => {
                    if *b {
                        evaluate_node(then_do.clone())
                    } else {
                        evaluate_node(else_do.clone())
                    }
                }
                _ => todo!(),
            }
        }
    }
}

fn substitute(node: Rc<Node>, var: VarId, value: Rc<Node>) -> Rc<Node> {
    match node.as_ref() {
        Node::Value(_) => node,
        Node::Lambda { var: v, body } => {
            if *v != var {
                Rc::new(Node::Lambda {
                    var: *v,
                    body: substitute(body.clone(), var, value),
                })
            } else {
                node
            }
        }
        Node::Variable(v) => {
            if *v == var {
                value
            } else {
                node
            }
        }
        Node::Apply { f, value: v } => Rc::new(Node::Apply {
            f: substitute(f.clone(), var, value.clone()),
            value: substitute(v.clone(), var, value),
        }),
        Node::BinaryOp { op, left, right } => Rc::new(Node::BinaryOp {
            op: *op,
            left: substitute(left.clone(), var, value.clone()),
            right: substitute(right.clone(), var, value),
        }),
        Node::UnuaryOp { op, body } => Rc::new(Node::UnuaryOp {
            op: *op,
            body: substitute(body.clone(), var, value),
        }),
        Node::If {
            cond,
            then_do,
            else_do,
        } => Rc::new(Node::If {
            cond: substitute(cond.clone(), var, value.clone()),
            then_do: substitute(then_do.clone(), var, value.clone()),
            else_do: substitute(else_do.clone(), var, value),
        }),
    }
}

#[cfg(test)]
mod tests {
    use logos::Logos;

    use crate::{
        ast::{BinaryOp, VarId},
        lexer::Token,
        parser::parse,
    };

    use super::*;

    #[test]
    fn example() {
        let tree = Rc::new(Node::Apply {
            f: Rc::new(Node::Apply {
                f: Rc::new(Node::Lambda {
                    var: VarId::new(2),
                    body: Rc::new(Node::Lambda {
                        var: VarId::new(3),
                        body: Rc::new(Node::Variable(VarId::new(2))),
                    }),
                }),
                value: Rc::new(Node::BinaryOp {
                    op: BinaryOp::StrConcat,
                    left: Rc::new(Node::Value(Value::Str("Hello".to_string()))),
                    right: Rc::new(Node::Value(Value::Str(" World!".to_string()))),
                }),
            }),
            value: Rc::new(Node::Value(Value::Int(42))),
        });
        assert_eq!(evaluate(tree), Value::Str("Hello World!".to_string()));
    }

    #[test]
    fn language_test() {
        const TASK: &str = "? B= B$ B$ B$ B$ L$ L$ L$ L# v$ I\" I# I$ I% I$ ? B= B$ L$ v$ I+ I+ ? B= BD I$ S4%34 S4 ? B= BT I$ S4%34 S4%3 ? B= B. S4% S34 S4%34 ? U! B& T F ? B& T T ? U! B| F F ? B| F T ? B< U- I$ U- I# ? B> I$ I# ? B= U- I\" B% U- I$ I# ? B= I\" B% I( I$ ? B= U- I\" B/ U- I$ I# ? B= I# B/ I( I$ ? B= I' B* I# I$ ? B= I$ B+ I\" I# ? B= U$ I4%34 S4%34 ? B= U# S4%34 I4%34 ? U! F ? B= U- I$ B- I# I& ? B= I$ B- I& I# ? B= S4%34 S4%34 ? B= F F ? B= I$ I$ ? T B. B. SM%,&k#(%#+}IEj}3%.$}z3/,6%},!.'5!'%y4%34} U$ B+ I# B* I$> I1~s:U@ Sz}4/}#,!)-}0/).43}&/2})4 S)&})3}./4}#/22%#4 S\").!29}q})3}./4}#/22%#4 S\").!29}q})3}./4}#/22%#4 S\").!29}q})3}./4}#/22%#4 S\").!29}k})3}./4}#/22%#4 S5.!29}k})3}./4}#/22%#4 S5.!29}_})3}./4}#/22%#4 S5.!29}a})3}./4}#/22%#4 S5.!29}b})3}./4}#/22%#4 S\").!29}i})3}./4}#/22%#4 S\").!29}h})3}./4}#/22%#4 S\").!29}m})3}./4}#/22%#4 S\").!29}m})3}./4}#/22%#4 S\").!29}c})3}./4}#/22%#4 S\").!29}c})3}./4}#/22%#4 S\").!29}r})3}./4}#/22%#4 S\").!29}p})3}./4}#/22%#4 S\").!29}{})3}./4}#/22%#4 S\").!29}{})3}./4}#/22%#4 S\").!29}d})3}./4}#/22%#4 S\").!29}d})3}./4}#/22%#4 S\").!29}l})3}./4}#/22%#4 S\").!29}N})3}./4}#/22%#4 S\").!29}>})3}./4}#/22%#4 S!00,)#!4)/.})3}./4}#/22%#4 S!00,)#!4)/.})3}./4}#/22%#4";
        let tree = parse(&mut Token::lexer(TASK)).unwrap();
        assert_eq!(evaluate(tree), Value::Bool(true));
    }
}
