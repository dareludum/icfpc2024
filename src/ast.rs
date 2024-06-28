use std::{fmt::Display, io::Write, rc::Rc};

use display_tree::{AsTree, DisplayTree};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Str(String),
    Int(i64),
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

    pub fn as_int(&self) -> i64 {
        match self {
            Value::Int(i) => *i,
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

#[derive(Clone, Debug, PartialEq, Eq, DisplayTree)]
pub enum Node {
    Value(#[node_label] Value),
    // even though lambda and apply technically are unuary / binary operators,
    // they are treated separately as they have to deal with scoping / evaluation
    Lambda {
        #[node_label]
        var: VarId,
        #[tree]
        body: NodeRef,
    },
    Variable(VarId),
    Apply {
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
    pub fn print(&self, f: &mut dyn std::io::Write) -> Result<(), std::io::Error> {
        writeln!(f, "{}", AsTree::new(self))
    }
}
