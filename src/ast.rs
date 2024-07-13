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
pub struct Decl;

#[derive(Debug)]
pub enum Stmt {
    Return {
        return_value: Box<Exp>
    }
}
