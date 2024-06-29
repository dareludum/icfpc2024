use std::{collections::HashMap, rc::Rc};

use crate::icfp::{Node, NodeRef, VarId};

use super::{ast::Binding, Iden, LNode, LNodeRef};

struct Compiler {
    // TODO: smart iden allocation?
    iden_count: u64,
    idens: HashMap<Iden, VarId>,
    y_combinator: Option<(VarId, NodeRef)>,
}

impl Compiler {
    fn new() -> Self {
        Self {
            iden_count: 0,
            idens: HashMap::new(),
            y_combinator: None,
        }
    }

    fn allocate_varid(&mut self) -> VarId {
        let id = VarId::new(self.iden_count);
        self.iden_count += 1;
        id
    }

    fn resolve(&mut self, iden: &Iden) -> VarId {
        if let Some(id) = self.idens.get(iden) {
            return *id;
        }

        let id = self.allocate_varid();
        self.idens.insert(iden.clone(), id);
        id
    }

    // the y combinator is only created once, and added at the top level
    fn get_y_combinator(&mut self) -> NodeRef {
        if let Some((id, _)) = self.y_combinator {
            return var(id);
        }

        // Lambda
        // ├── VarId(1)
        // └── Apply
        //     ├── Lambda
        //     │   ├── VarId(2)
        //     │   └── Apply
        //     │       ├── VarId(1)
        //     │       └── Apply
        //     │           ├── VarId(2)
        //     │           └── VarId(2)
        //     └── Lambda
        //         ├── VarId(2)
        //         └── Apply
        //             ├── VarId(1)
        //             └── Apply
        //                 ├── VarId(2)
        //                 └── VarId(2)
        let v1 = self.allocate_varid();
        let v2 = self.allocate_varid();
        let cc = lambda(v2, apply(var(v1), apply(var(v2), var(v2))));
        let node = lambda(v1, apply(cc.clone(), cc));

        let id = self.allocate_varid();
        self.y_combinator = Some((id, node));
        var(id)
    }

    // simplify a binding
    fn compile_binding(&mut self, binding: &Binding) -> (VarId, NodeRef) {
        let var_id = self.resolve(&binding.name);
        let mut body = self.compile_node(&binding.value);

        // if the binding is a variable
        if binding.params.is_empty() {
            assert!(!binding.rec);
            return (var_id, body);
        }

        // if it is a function, bind parameters
        for param in binding.params.iter().rev() {
            body = lambda(self.resolve(param), body)
        }

        // if the function is recursive, apply the Y combinator to a lambda of the binding's name
        body = apply(self.get_y_combinator(), lambda(var_id, body));
        (var_id, body)
    }

    fn compile_node(&mut self, source: &LNode) -> NodeRef {
        Rc::new(match source {
            super::LNode::Litteral(val) => Node::Value(val.clone()),
            super::LNode::Variable(var) => Node::Variable(self.resolve(var)),
            super::LNode::Apply { func, param } => Node::Apply {
                f: self.compile_node(func),
                value: self.compile_node(param),
            },
            super::LNode::UnuaryOp { op, body } => Node::UnuaryOp {
                op: *op,
                body: self.compile_node(body),
            },
            super::LNode::BinaryOp { op, left, right } => Node::BinaryOp {
                op: *op,
                left: self.compile_node(left),
                right: self.compile_node(right),
            },
            super::LNode::If {
                cond,
                then_do,
                else_do,
            } => Node::If {
                cond: self.compile_node(cond),
                then_do: self.compile_node(then_do),
                else_do: self.compile_node(else_do),
            },
            super::LNode::Let { bindings, body } => {
                let bindings: Vec<_> = bindings
                    .iter()
                    .map(|binding| self.compile_binding(binding))
                    .collect();
                let body = self.compile_node(body);
                return bindings
                    .iter()
                    .rev()
                    .fold(body, |acc, (var_id, var_value)| {
                        bind(*var_id, var_value.clone(), acc)
                    });
            }
        })
    }

    pub fn compile(&mut self, source: LNodeRef) -> NodeRef {
        let res = self.compile_node(source.as_ref());
        if let Some((id, node)) = &self.y_combinator {
            bind(*id, node.clone(), res)
        } else {
            res
        }
    }
}

pub fn compile(source: LNodeRef) -> NodeRef {
    Compiler::new().compile(source)
}

fn var(id: VarId) -> NodeRef {
    Rc::new(Node::Variable(id))
}

fn apply(f: NodeRef, value: NodeRef) -> NodeRef {
    Rc::new(Node::Apply { f, value })
}

fn lambda(var: VarId, body: NodeRef) -> NodeRef {
    Rc::new(Node::Lambda { var, body })
}

fn bind(var: VarId, value: NodeRef, body: NodeRef) -> NodeRef {
    apply(lambda(var, body), value)
}

#[cfg(test)]
mod tests {
    use std::io::stdout;

    use crate::icfp::evaluate;

    use super::super::parse;
    use super::compile;

    #[test]
    fn test_let_in() {
        let sample = r#"
            let a = 1;
                b = a + 1;
                f x y = x * y + b;
            in (f 2 a)
        "#;
        let node = parse(sample).unwrap();
        let node = compile(node);
        assert_eq!(evaluate(node).as_int(), 4);
    }

    #[test]
    fn test_rec() {
        let sample = r#"
            let rec fac x = if x < 2 { x } else { x * (fac (x - 1)) };
            in fac 3
        "#;
        let node = parse(sample).unwrap();
        let node = compile(node);
        let _ = node.pretty_print(&mut stdout().lock());
        assert_eq!(evaluate(node).as_int(), 6);
    }
}
