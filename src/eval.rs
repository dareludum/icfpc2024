use crate::ast::{Node, Value};

pub fn evaluate(tree: Node) -> Value {
    todo!()
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::ast::{BinaryOp, VarId};

    use super::*;

    #[test]
    fn example() {
        let tree = Node::Apply {
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
        };
        assert_eq!(evaluate(tree), Value::Str("Hello World!".to_string()));
    }
}
