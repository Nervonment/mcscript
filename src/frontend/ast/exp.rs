use super::{DataType, Ident, SrcLocation};

#[derive(Debug, Clone)]
pub struct Exp {
    pub exp_type: ExpType,
    pub src_loc: SrcLocation,
}

#[derive(Debug, Clone)]
pub enum ExpType {
    UnaryExp(UnaryOp, Box<Exp>),
    BinaryExp(BinaryOp, Box<Exp>, Box<Exp>),
    Number(i32),
    Variable {
        ident: Ident,
        namespace: Option<Ident>,
    },
    ArrayElement {
        array: Box<Exp>,
        subscript: Box<Exp>,
    },
    FuncCall {
        namespace: Option<Ident>,
        func_ident: Ident,
        arguments: Vec<Box<Exp>>,
    },
    NewArray {
        length: Box<Exp>,
        element: Box<Exp>,
    },
    SquareBracketsArray {
        element_type: Option<DataType>,
        elements: Vec<Box<Exp>>,
    },
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Positive,
    Negative,
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Ne,
    LAnd,
    LOr,
}
