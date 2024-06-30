use std::{collections::HashMap, rc::Rc};

use crate::icfp::{EvalStrat, Node, NodeRef, VarId};

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
            return Node::var(id);
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
        let cc = {
            let body = {
                let f = Node::var(v1);
                let value = Node::apply(EvalStrat::Name, Node::var(v2), Node::var(v2));
                Node::apply(EvalStrat::Name, f, value)
            };
            Node::lambda(v2, body)
        };
        let node = {
            let body = {
                let f = cc.clone();
                Node::apply(EvalStrat::Name, f, cc)
            };
            Node::lambda(v1, body)
        };

        let id = self.allocate_varid();
        self.y_combinator = Some((id, node));
        Node::var(id)
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
            body = {
                let var = self.resolve(param);
                Node::lambda(var, body)
            }
        }

        // if the function is recursive, apply the Y combinator to a lambda of the binding's name
        body = {
            let f = self.get_y_combinator();
            let value = Node::lambda(var_id, body);
            Node::apply(EvalStrat::Value, f, value)
        };
        (var_id, body)
    }

    fn compile_node(&mut self, source: &LNode) -> NodeRef {
        Rc::new(match source {
            super::LNode::Litteral(val) => Node::Value(val.clone()),
            super::LNode::Variable(var) => Node::Variable(self.resolve(var)),
            super::LNode::Apply { func, param } => Node::Apply {
                strat: EvalStrat::Name,
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
                        let var = *var_id;
                        let value = var_value.clone();
                        Node::bind(var, value, acc)
                    });
            }
        })
    }

    pub fn compile(&mut self, source: LNodeRef) -> NodeRef {
        let res = self.compile_node(source.as_ref());
        if let Some((id, node)) = &self.y_combinator {
            {
                let var = *id;
                let value = node.clone();
                Node::bind(var, value, res)
            }
        } else {
            res
        }
    }
}

pub fn compile(source: LNodeRef) -> NodeRef {
    Compiler::new().compile(source)
}
