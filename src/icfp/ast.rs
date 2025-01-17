use std::{fmt::Display, rc::Rc};

use display_tree::{AsTree, DisplayTree};

use super::base94::Base94Int;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Str(String),
    Int(Base94Int),
    Bool(bool),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Str(v) => write!(f, "{:?}", v),
            Value::Int(v) => write!(f, "{:?}", v),
            Value::Bool(v) => write!(f, "{:?}", v),
        }
    }
}

impl Value {
    pub fn as_str(&self) -> &str {
        match self {
            Value::Str(s) => s,
            _ => panic!("Expected string"),
        }
    }

    pub fn as_int(&self) -> &Base94Int {
        match self {
            Value::Int(i) => i,
            _ => panic!("Expected int"),
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            _ => panic!("Expected bool"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct VarId(u64);

impl VarId {
    pub fn new(id: u64) -> Self {
        VarId(id)
    }

    pub fn id(&self) -> u64 {
        self.0
    }
}

impl Display for VarId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VarId({})", self.0)
    }
}

pub type NodeRef = Rc<Node>;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    IntAdd,
    IntSub,
    IntMul,
    IntDiv,
    IntMod,
    IntLt,
    IntGt,
    BoolOr,
    BoolAnd,
    StrConcat,
    StrTake,
    StrDrop,
    Eq,
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum UnuaryOp {
    IntNeg,
    BoolNot,
    StrToInt,
    IntToStr,
}

impl Display for UnuaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvalStrat {
    Name,
    Value,
    Lazy,
}

impl Display for EvalStrat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, DisplayTree)]
pub enum Node {
    Value(#[node_label] Value),
    // even though lambda and apply technically are unuary / binary operators,
    // they are treated separately as they have to deal with scoping / evaluation
    Lambda {
        var: VarId,
        #[tree]
        body: NodeRef,
    },
    Variable(#[node_label] VarId),
    Apply {
        #[node_label]
        strat: EvalStrat,
        #[tree]
        f: NodeRef,
        #[tree]
        value: NodeRef,
    },
    BinaryOp {
        #[node_label]
        op: BinaryOp,
        #[tree]
        left: NodeRef,
        #[tree]
        right: NodeRef,
    },
    UnuaryOp {
        op: UnuaryOp,
        #[tree]
        body: NodeRef,
    },
    If {
        #[tree]
        #[field_label]
        cond: NodeRef,
        #[tree]
        #[field_label]
        then_do: NodeRef,
        #[tree]
        #[field_label]
        else_do: NodeRef,
    },
}

impl Node {
    pub fn pretty_print(&self, f: &mut dyn std::io::Write) -> Result<(), std::io::Error> {
        writeln!(f, "{}", AsTree::new(self))
    }

    pub fn var(id: VarId) -> NodeRef {
        Rc::new(Node::Variable(id))
    }

    pub fn apply(strat: EvalStrat, f: NodeRef, value: NodeRef) -> NodeRef {
        Rc::new(Node::Apply { strat, f, value })
    }

    pub fn lambda(var: VarId, body: NodeRef) -> NodeRef {
        Rc::new(Node::Lambda { var, body })
    }

    pub fn bind(var: VarId, value: NodeRef, body: NodeRef) -> NodeRef {
        Self::apply(EvalStrat::Value, Self::lambda(var, body), value)
    }
}
