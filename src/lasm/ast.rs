use std::rc::Rc;

pub use crate::icfp::{BinaryOp, UnuaryOp, Value};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Iden(String);

impl Iden {
    pub fn new(name: String) -> Self {
        Iden(name)
    }
}

impl From<String> for Iden {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

pub type LNodeRef = Rc<LNode>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Binding {
    pub rec: bool,
    pub name: Iden,
    pub params: Vec<Iden>,
    pub value: LNodeRef,
}

impl Binding {
    pub fn new(rec: bool, name: Iden, params: Vec<Iden>, value: LNodeRef) -> Self {
        Binding {
            rec,
            name,
            params,
            value,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LNode {
    Litteral(Value),
    Let {
        bindings: Vec<Binding>,
        body: LNodeRef,
    },
    Variable(Iden),
    Apply {
        func: LNodeRef,
        param: LNodeRef,
    },
    BinaryOp {
        op: BinaryOp,
        left: LNodeRef,
        right: LNodeRef,
    },
    UnuaryOp {
        op: UnuaryOp,
        body: LNodeRef,
    },
    If {
        cond: LNodeRef,
        then_do: LNodeRef,
        else_do: LNodeRef,
    },
}

impl LNode {
    pub fn cond(cond: LNodeRef, then_do: LNodeRef, else_do: LNodeRef) -> LNodeRef {
        Rc::new(Self::If {
            cond,
            then_do,
            else_do,
        })
    }

    pub fn binary_op(op: BinaryOp, left: LNodeRef, right: LNodeRef) -> LNodeRef {
        Rc::new(Self::BinaryOp { op, left, right })
    }

    pub fn unuary_op(op: UnuaryOp, body: LNodeRef) -> LNodeRef {
        Rc::new(Self::UnuaryOp { op, body })
    }

    pub fn apply(func: LNodeRef, param: LNodeRef) -> LNodeRef {
        Rc::new(Self::Apply { func, param })
    }

    pub fn var(name: impl Into<Iden>) -> LNodeRef {
        Rc::new(Self::Variable(name.into()))
    }

    pub fn value(val: Value) -> LNodeRef {
        Rc::new(Self::Litteral(val))
    }

    pub fn int(val: i64) -> LNodeRef {
        Self::value(Value::Int(val))
    }

    pub fn bool(val: bool) -> LNodeRef {
        Self::value(Value::Bool(val))
    }

    pub fn str(val: impl Into<String>) -> LNodeRef {
        Self::value(Value::Str(val.into()))
    }
}
