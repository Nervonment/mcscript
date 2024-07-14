use exp::Exp;

pub mod exp;

#[derive(Debug)]
pub struct Program {
    pub func_def: FuncDef,
}

#[derive(Debug)]
pub struct FuncDef {
    pub ident: String,
    pub block: Block,
}

#[derive(Debug)]
pub struct Block(pub Vec<BlockItem>);

#[derive(Debug)]
pub enum BlockItem {
    Decl(Decl),
    Stmt(Stmt),
}

#[derive(Debug)]
pub struct Decl {
    pub ident: String,
    pub init_value: Box<Exp>,
}

#[derive(Debug)]
pub enum Stmt {
    Return {
        return_value: Box<Exp>,
    },
    Assign {
        ident: String,
        new_value: Box<Exp>,
    },
    Block(Block),
    // IfElse {
    //     exp: Box<Exp>,
    //     if_branch: Box<Stmt>,
    //     else_branch: Option<Box<Stmt>>,
    // },
}
