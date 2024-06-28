use std::rc::Rc;

#[derive(Clone, Debug)]
enum Value {
    Str(String),
    Int(isize),
    Bool(bool),
}

#[derive(Copy, Clone, Debug)]
struct VarId(usize);

type NodeRef = Rc<Node>;

#[derive(Copy, Clone, Debug)]
enum BinaryOp {
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
enum UnuaryOp {
    IntNeg,
    BoolNot,
    StrToInt,
    IntToStr,
}

#[derive(Clone, Debug)]
enum Node {
    Value(Value),
    // even though lambda and apply technically are unuary / binary operators,
    // they are treated separately as they have to deal with scoping / evaluation
    Lambda {
        var: VarId,
        body: NodeRef,
    },
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
