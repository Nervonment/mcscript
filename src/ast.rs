use exp::Exp;

pub mod exp;

#[derive(Debug)]
pub struct Program {
    pub func_defs: Vec<FuncDef>,
}

#[derive(Debug)]
pub struct FuncDef {
    pub ident: String,
    pub params: Vec<FuncParam>,
    pub block: Block,
}

#[derive(Debug, Clone)]
pub struct FuncParam {
    pub ident: String,
}

#[derive(Debug, Clone)]
pub struct Block(pub Vec<BlockItem>);

#[derive(Debug, Clone)]
pub enum BlockItem {
    Decl(Decl),
    Stmt(Stmt),
}

#[derive(Debug, Clone)]
pub struct Decl {
    pub ident: String,
    pub init_value: Box<Exp>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Return {
        return_value: Box<Exp>,
    },
    Assign {
        ident: String,
        new_value: Box<Exp>,
    },
    Block(Block),
    IfElse {
        exp: Box<Exp>,
        if_branch: Block,
        else_branch: Option<Block>,
    },
    While {
        exp: Box<Exp>,
        body: Block,
    },
    Break,
    Continue,
    Exp(Box<Exp>),
}
