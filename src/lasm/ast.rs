use std::rc::Rc;

pub use crate::icfp::{BinaryOp, UnuaryOp, Value};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Iden(String);

impl Iden {
    pub fn new(name: String) -> Self {
        Iden(name)
    }
}

pub type LNodeRef = Rc<LNode>;

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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
