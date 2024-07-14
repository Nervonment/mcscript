#[derive(Debug, Clone)]
pub enum Exp {
    UnaryExp(UnaryOp, Box<Exp>),
    BinaryExp(BinaryOp, Box<Exp>, Box<Exp>),
    Number(i32),
    Variable(String),
    FuncCall {
        func_ident: String,
        arguments: Vec<Box<Exp>>,
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
}
