use exp::Exp;

pub mod exp;

#[derive(Debug, Clone)]
pub struct SrcLocation {
    pub begin: usize,
    pub end: usize,
}

#[derive(Debug)]
pub struct CompileUnit {
    pub global_defs: Vec<GlobalDef>,
}

#[derive(Debug)]
pub enum GlobalDef {
    FuncDef(FuncDef),
    VariableDef {
        ident: Ident,
        init_value: Box<Exp>,
        data_type: DataType,
    },
}

#[derive(Debug, Clone)]
pub struct FuncDef {
    pub ident: Ident,
    pub params: Vec<FuncParam>,
    pub block: Block,
    pub func_type: Option<DataType>,
}

#[derive(Debug, Clone)]
pub struct FuncParam {
    pub ident: Ident,
    pub data_type: DataType,
}

#[derive(Debug, Clone)]
pub struct Ident {
    pub string: String,
    pub src_loc: SrcLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Int,
    Array { element_type: Box<DataType> },
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
    pub ident: Ident,
    pub init_value: Box<Exp>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Return {
        return_value: Option<Box<Exp>>,
    },
    Assign {
        lhs: Box<Exp>,
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
    InlineCommand {
        fmt_str: String,
        arguments: Vec<Box<Exp>>,
    },
}
