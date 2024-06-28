use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum Value {
    Str(String),
    Int(i64),
    Bool(bool),
}

#[derive(Copy, Clone, Debug)]
pub struct VarId(u64);

impl VarId {
    pub fn new(id: u64) -> Self {
        VarId(id)
    }
}

pub type NodeRef = Rc<Node>;

#[derive(Copy, Clone, Debug)]
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

#[derive(Copy, Clone, Debug)]
pub enum UnuaryOp {
    IntNeg,
    BoolNot,
    StrToInt,
    IntToStr,
}

#[derive(Clone, Debug)]
pub enum Node {
    Value(Value),
    // even though lambda and apply technically are unuary / binary operators,
    // they are treated separately as they have to deal with scoping / evaluation
    Lambda {
        var: VarId,
        body: NodeRef,
    },
    Variable(VarId),
    Apply {
        f: NodeRef,
        value: NodeRef,
    },
    BinaryOp {
        op: BinaryOp,
        left: NodeRef,
        right: NodeRef,
    },
    UnuaryOp {
        op: UnuaryOp,
        body: NodeRef,
    },
    If {
        cond: NodeRef,
        then_do: NodeRef,
        else_do: NodeRef,
    },
}
