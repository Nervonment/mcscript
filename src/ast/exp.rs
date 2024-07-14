#[derive(Debug)]
pub enum Exp {
    UnaryExp(UnaryOp, Box<Exp>),
    BinaryExp(BinaryOp, Box<Exp>, Box<Exp>),
    Number(i32),
    Variable(String),
}

#[derive(Debug)]
pub enum UnaryOp {
    Positive,
    Negative,
}

#[derive(Debug)]
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