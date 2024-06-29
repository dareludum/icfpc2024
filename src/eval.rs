use std::rc::Rc;

use crate::ast::{BinaryOp, Node, NodeRef, UnuaryOp, Value, VarId};

pub fn evaluate(tree: Rc<Node>) -> Value {
    Evaluator::new().evaluate(tree)
}

struct Evaluator {
    num_substitutions: u32,
}

impl Evaluator {
    fn new() -> Self {
        Self {
            num_substitutions: 0,
        }
    }

    fn evaluate(&mut self, mut tree: Rc<Node>) -> Value {
        let mut strict_reductions = 0;
        loop {
            let current_substitutions = self.num_substitutions;
            tree = self.beta_reduction(tree.clone());

            loop {
                let (new_tree, reduced) = Self::strict_reduction(tree.clone());
                if reduced {
                    tree = new_tree;
                    strict_reductions += 1;
                } else {
                    break;
                }
                if strict_reductions > 10_000_000 {
                    panic!("Too many strict reductions");
                }
            }

            if self.num_substitutions == current_substitutions {
                if let Node::Value(v) = tree.as_ref() {
                    return v.clone();
                } else {
                    panic!("Didn't reduce to a value");
                }
            } else if self.num_substitutions > 10_000_000 {
                panic!("Too many substitutions");
            }
        }
    }

    // Performs a beta reduction on the tree
    fn beta_reduction(&mut self, tree: Rc<Node>) -> Rc<Node> {
        match tree.as_ref() {
            Node::Value(_) => tree,
            Node::Lambda { .. } => tree,
            Node::Variable(_) => panic!("Unsubstituted variable"),
            Node::Apply { f, value } => {
                let node = self.beta_reduction(f.clone());
                match node.as_ref() {
                    Node::Lambda { var, body } => {
                        let node = Self::substitute(body.clone(), *var, value.clone());
                        self.num_substitutions += 1;
                        node
                    }
                    _ => Rc::new(Node::Apply {
                        f: node,
                        value: self.beta_reduction(value.clone()),
                    }),
                }
            }
            Node::BinaryOp { op, left, right } => Rc::new(Node::BinaryOp {
                op: *op,
                left: self.beta_reduction(left.clone()),
                right: self.beta_reduction(right.clone()),
            }),
            Node::UnuaryOp { op, body } => Rc::new(Node::UnuaryOp {
                op: *op,
                body: self.beta_reduction(body.clone()),
            }),
            Node::If {
                cond,
                then_do,
                else_do,
            } => Rc::new(Node::If {
                cond: self.beta_reduction(cond.clone()),
                then_do: self.beta_reduction(then_do.clone()),
                else_do: self.beta_reduction(else_do.clone()),
            }),
        }
    }

    // Computes strict nodes and folds
    fn strict_reduction(tree: Rc<Node>) -> (Rc<Node>, bool) {
        match tree.as_ref() {
            Node::Value(_) => (tree, false),
            Node::Lambda { var, body } => {
                let (body, reduced) = Self::strict_reduction(body.clone());
                if reduced {
                    (
                        Rc::new(Node::Lambda {
                            var: *var,
                            body: body.clone(),
                        }),
                        true,
                    )
                } else {
                    (tree, false)
                }
            }
            Node::Variable(_) => (tree, false),
            Node::Apply { f, value } => {
                let (f, reduced_f) = Self::strict_reduction(f.clone());
                let (value, reduced_value) = Self::strict_reduction(value.clone());
                if reduced_f || reduced_value {
                    (
                        Rc::new(Node::Apply {
                            f: f.clone(),
                            value: value.clone(),
                        }),
                        true,
                    )
                } else {
                    (tree, false)
                }
            }
            Node::BinaryOp { op, left, right } => {
                let (left, reduced_left) = Self::strict_reduction(left.clone());
                let (right, reduced_right) = Self::strict_reduction(right.clone());
                if let (Node::Value(l), Node::Value(r)) = (left.as_ref(), right.as_ref()) {
                    match op {
                        BinaryOp::IntAdd => (int(l.as_int() + r.as_int()), true),
                        BinaryOp::IntSub => (int(l.as_int() - r.as_int()), true),
                        BinaryOp::IntMul => (int(l.as_int() * r.as_int()), true),
                        BinaryOp::IntDiv => (int(l.as_int() / r.as_int()), true),
                        BinaryOp::IntMod => (int(l.as_int() % r.as_int()), true),
                        BinaryOp::IntLt => (bool(l.as_int() < r.as_int()), true),
                        BinaryOp::IntGt => (bool(l.as_int() > r.as_int()), true),
                        BinaryOp::BoolOr => (bool(l.as_bool() || r.as_bool()), true),
                        BinaryOp::BoolAnd => (bool(l.as_bool() && r.as_bool()), true),
                        BinaryOp::StrConcat => (str(format!("{}{}", l.as_str(), r.as_str())), true),
                        BinaryOp::StrTake => (
                            str(r.as_str().chars().take(l.as_int() as usize).collect()),
                            true,
                        ),
                        BinaryOp::StrDrop => (
                            str(r.as_str().chars().skip(l.as_int() as usize).collect()),
                            true,
                        ),
                        BinaryOp::Eq => (bool(l == r), true),
                    }
                } else {
                    if let (
                        Node::Value(l),
                        Node::BinaryOp {
                            op: op2,
                            left: right,
                            right: next,
                        },
                    ) = (left.as_ref(), right.as_ref())
                    {
                        if let Node::Value(r) = right.as_ref() {
                            if *op == *op2 {
                                match op {
                                    BinaryOp::IntAdd => {
                                        return (
                                            Rc::new(Node::BinaryOp {
                                                op: *op,
                                                left: int(l.as_int() + r.as_int()),
                                                right: next.clone(),
                                            }),
                                            true,
                                        );
                                    }
                                    BinaryOp::IntMul => {
                                        return (
                                            Rc::new(Node::BinaryOp {
                                                op: *op,
                                                left: int(l.as_int() * r.as_int()),
                                                right: next.clone(),
                                            }),
                                            true,
                                        );
                                    }
                                    BinaryOp::BoolAnd => {
                                        return (
                                            Rc::new(Node::BinaryOp {
                                                op: *op,
                                                left: bool(l.as_bool() && r.as_bool()),
                                                right: next.clone(),
                                            }),
                                            true,
                                        );
                                    }
                                    BinaryOp::BoolOr => {
                                        return (
                                            Rc::new(Node::BinaryOp {
                                                op: *op,
                                                left: bool(l.as_bool() || r.as_bool()),
                                                right: next.clone(),
                                            }),
                                            true,
                                        );
                                    }
                                    BinaryOp::StrConcat => {
                                        return (
                                            Rc::new(Node::BinaryOp {
                                                op: *op,
                                                left: str(format!("{}{}", l.as_str(), r.as_str())),
                                                right: next.clone(),
                                            }),
                                            true,
                                        );
                                    }
                                    _ => todo!(),
                                }
                            }
                        }
                    }

                    if reduced_left || reduced_right {
                        (
                            Rc::new(Node::BinaryOp {
                                op: *op,
                                left,
                                right,
                            }),
                            true,
                        )
                    } else {
                        (tree, false)
                    }
                }
            }
            Node::UnuaryOp { op, body } => {
                let (body, reduced) = Self::strict_reduction(body.clone());
                if let Node::Value(v) = body.as_ref() {
                    match op {
                        UnuaryOp::IntNeg => (int(-v.as_int()), true),
                        UnuaryOp::BoolNot => (bool(!v.as_bool()), true),
                        UnuaryOp::StrToInt => (
                            int(crate::base94::base94_to_int(&crate::base94::str_to_base94(
                                v.as_str(),
                            ))
                            .unwrap() as i64),
                            true,
                        ),
                        UnuaryOp::IntToStr => (
                            str(crate::base94::base94_to_str(&crate::base94::int_to_base94(
                                v.as_int() as u64,
                            ))),
                            true,
                        ),
                    }
                } else if reduced {
                    (Rc::new(Node::UnuaryOp { op: *op, body }), true)
                } else {
                    (tree, false)
                }
            }
            Node::If {
                cond,
                then_do,
                else_do,
            } => {
                let (cond, reduced) = Self::strict_reduction(cond.clone());
                if let Node::Value(Value::Bool(b)) = cond.as_ref() {
                    if *b {
                        let (then_do, _) = Self::strict_reduction(then_do.clone());
                        (then_do, true)
                    } else {
                        let (else_do, _) = Self::strict_reduction(else_do.clone());
                        (else_do, true)
                    }
                } else if reduced {
                    (
                        Rc::new(Node::If {
                            cond,
                            then_do: then_do.clone(),
                            else_do: else_do.clone(),
                        }),
                        true,
                    )
                } else {
                    (tree, false)
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
                        body: Self::substitute(body.clone(), var, value),
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
                f: Self::substitute(f.clone(), var, value.clone()),
                value: Self::substitute(v.clone(), var, value),
            }),
            Node::BinaryOp { op, left, right } => Rc::new(Node::BinaryOp {
                op: *op,
                left: Self::substitute(left.clone(), var, value.clone()),
                right: Self::substitute(right.clone(), var, value),
            }),
            Node::UnuaryOp { op, body } => Rc::new(Node::UnuaryOp {
                op: *op,
                body: Self::substitute(body.clone(), var, value),
            }),
            Node::If {
                cond,
                then_do,
                else_do,
            } => Rc::new(Node::If {
                cond: Self::substitute(cond.clone(), var, value.clone()),
                then_do: Self::substitute(then_do.clone(), var, value.clone()),
                else_do: Self::substitute(else_do.clone(), var, value),
            }),
        }
    }
}

fn int(v: i64) -> NodeRef {
    Rc::new(Node::Value(Value::Int(v)))
}

fn bool(v: bool) -> NodeRef {
    Rc::new(Node::Value(Value::Bool(v)))
}

fn str(v: String) -> NodeRef {
    Rc::new(Node::Value(Value::Str(v)))
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

    fn eval(code: &str) -> Value {
        evaluate(parse(&mut Token::lexer(code)).unwrap())
    }

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
    fn unary_negate() {
        const TASK: &str = "U- I$";
        assert_eq!(eval(TASK), Value::Int(-3));
    }

    #[test]
    fn unary_not() {
        const TASK: &str = "U! T";
        assert_eq!(eval(TASK), Value::Bool(false));
    }

    #[test]
    fn unary_string_to_int() {
        const TASK: &str = "U# S4%34";
        assert_eq!(eval(TASK), Value::Int(15818151));
    }

    #[test]
    fn unary_int_to_string() {
        const TASK: &str = "U$ I4%34";
        assert_eq!(eval(TASK), Value::Str("test".to_string()));
    }

    #[test]
    fn binary_add() {
        const TASK: &str = "B+ I# I$";
        assert_eq!(eval(TASK), Value::Int(5));
    }

    #[test]
    fn binary_sub() {
        const TASK: &str = "B- I$ I#";
        assert_eq!(eval(TASK), Value::Int(1));
    }

    #[test]
    fn binary_mul() {
        const TASK: &str = "B* I$ I#";
        assert_eq!(eval(TASK), Value::Int(6));
    }

    #[test]
    fn binary_div() {
        const TASK: &str = "B/ U- I( I#";
        assert_eq!(eval(TASK), Value::Int(-3));
    }

    #[test]
    fn binary_mod() {
        const TASK: &str = "B% U- I( I#";
        assert_eq!(eval(TASK), Value::Int(-1));
    }

    #[test]
    fn binary_lt() {
        const TASK: &str = "B< I$ I#";
        assert_eq!(eval(TASK), Value::Bool(false));
    }

    #[test]
    fn binary_gt() {
        const TASK: &str = "B> I$ I#";
        assert_eq!(eval(TASK), Value::Bool(true));
    }

    #[test]
    fn binary_eq() {
        const TASK: &str = "B= I$ I#";
        assert_eq!(eval(TASK), Value::Bool(false));
    }

    #[test]
    fn binary_or() {
        const TASK: &str = "B| T F";
        assert_eq!(eval(TASK), Value::Bool(true));
    }

    #[test]
    fn binary_and() {
        const TASK: &str = "B& T F";
        assert_eq!(eval(TASK), Value::Bool(false));
    }

    #[test]
    fn binary_str_concat() {
        const TASK: &str = "B. S4% S34";
        assert_eq!(eval(TASK), Value::Str("test".to_string()));
    }

    #[test]
    fn binary_str_take() {
        const TASK: &str = "BT I$ S4%34";
        assert_eq!(eval(TASK), Value::Str("tes".to_string()));
    }

    #[test]
    fn binary_str_drop() {
        const TASK: &str = "BD I$ S4%34";
        assert_eq!(eval(TASK), Value::Str("t".to_string()));
    }

    #[test]
    fn if_then_else() {
        const TASK: &str = "? B> I# I$ S9%3 S./";
        assert_eq!(eval(TASK), Value::Str("no".to_string()));
    }

    #[test]
    fn lambda() {
        const TASK: &str = "B$ B$ L# L$ v# B. SB%,,/ S}Q/2,$_ IK";
        assert_eq!(eval(TASK), Value::Str("Hello World!".to_string()));
    }

    #[test]
    fn num_substitutions() {
        const TASK: &str = "B$ B$ L\" B$ L# B$ v\" B$ v# v# L# B$ v\" B$ v# v# L\" L# ? B= v# I! I\" B$ L$ B+ B$ v\" v$ B$ v\" v$ B- v# I\" I%";
        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(parse(&mut Token::lexer(TASK)).unwrap());
        assert_eq!(result, Value::Int(16));
        assert_eq!(evaluator.num_substitutions, 109);
    }

    #[test]
    fn language_test() {
        const TASK: &str = "? B= B$ B$ B$ B$ L$ L$ L$ L# v$ I\" I# I$ I% I$ ? B= B$ L$ v$ I+ I+ ? B= BD I$ S4%34 S4 ? B= BT I$ S4%34 S4%3 ? B= B. S4% S34 S4%34 ? U! B& T F ? B& T T ? U! B| F F ? B| F T ? B< U- I$ U- I# ? B> I$ I# ? B= U- I\" B% U- I$ I# ? B= I\" B% I( I$ ? B= U- I\" B/ U- I$ I# ? B= I# B/ I( I$ ? B= I' B* I# I$ ? B= I$ B+ I\" I# ? B= U$ I4%34 S4%34 ? B= U# S4%34 I4%34 ? U! F ? B= U- I$ B- I# I& ? B= I$ B- I& I# ? B= S4%34 S4%34 ? B= F F ? B= I$ I$ ? T B. B. SM%,&k#(%#+}IEj}3%.$}z3/,6%},!.'5!'%y4%34} U$ B+ I# B* I$> I1~s:U@ Sz}4/}#,!)-}0/).43}&/2})4 S)&})3}./4}#/22%#4 S\").!29}q})3}./4}#/22%#4 S\").!29}q})3}./4}#/22%#4 S\").!29}q})3}./4}#/22%#4 S\").!29}k})3}./4}#/22%#4 S5.!29}k})3}./4}#/22%#4 S5.!29}_})3}./4}#/22%#4 S5.!29}a})3}./4}#/22%#4 S5.!29}b})3}./4}#/22%#4 S\").!29}i})3}./4}#/22%#4 S\").!29}h})3}./4}#/22%#4 S\").!29}m})3}./4}#/22%#4 S\").!29}m})3}./4}#/22%#4 S\").!29}c})3}./4}#/22%#4 S\").!29}c})3}./4}#/22%#4 S\").!29}r})3}./4}#/22%#4 S\").!29}p})3}./4}#/22%#4 S\").!29}{})3}./4}#/22%#4 S\").!29}{})3}./4}#/22%#4 S\").!29}d})3}./4}#/22%#4 S\").!29}d})3}./4}#/22%#4 S\").!29}l})3}./4}#/22%#4 S\").!29}N})3}./4}#/22%#4 S\").!29}>})3}./4}#/22%#4 S!00,)#!4)/.})3}./4}#/22%#4 S!00,)#!4)/.})3}./4}#/22%#4";
        let tree = parse(&mut Token::lexer(TASK)).unwrap();
        assert_eq!(
            evaluate(tree),
            Value::Str(
                "Self-check OK, send `solve language_test 4w3s0m3` to claim points for it"
                    .to_owned()
            )
        );
    }

    #[test]
    fn lambdaman10() {
        const TASK: &str = "B. SF B$ B$ L\" B$ L# B$ v\" B$ v# v# L# B$ v\" B$ v# v# L\" L# ? B= v# I;Y S B. ? B= B% v# IS I! S~ S B. ? B= B% v# I, I! Sa Sl B$ v\" B+ v# I\" I\"";
        // Make sure this doesn't overflow stack
        eval(TASK);
    }
}
