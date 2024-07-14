use std::collections::HashMap;

use crate::{
    ast::{
        exp::{BinaryOp, Exp, UnaryOp},
        Block, BlockItem, FuncDef, Program, Stmt,
    },
    datapack::{Datapack, Mcfunction, Namespace},
};

#[derive(Clone)]
pub struct Symbol {
    pub decorated_name: String,
}

pub struct SymbolTable(Vec<HashMap<String, Symbol>>);

impl SymbolTable {
    pub fn new() -> Self {
        Self(vec![HashMap::<String, Symbol>::new()])
    }

    pub fn enter_scope(&mut self) {
        self.0.push(HashMap::<_, _>::new());
    }

    pub fn leave_scope(&mut self) {
        self.0.pop();
    }

    pub fn new_symbol(&mut self, ident: &str) -> Symbol {
        if self.0.last().unwrap().contains_key(ident) {
            panic!();
        }
        let mut decorated_name = ident.to_owned();
        decorated_name.push_str(&format!("@{}", self.0.len() - 1));
        let symbol = Symbol {
            decorated_name: decorated_name.clone(),
        };
        self.0
            .last_mut()
            .unwrap()
            .insert(ident.to_owned(), symbol.clone());
        symbol
    }

    pub fn query_symbol(&self, ident: &str) -> (bool, Symbol) {
        for (level, scope) in self.0.iter().enumerate().rev() {
            if scope.contains_key(ident) {
                return (level > 0, scope[ident].clone());
            }
        }
        panic!();
    }
}

impl FuncDef {
    pub fn to_mcfunction(&self, dest: &mut Namespace, symbol_table: &mut SymbolTable) {
        let mut entry = Mcfunction::new(self.ident.clone());
        entry.append_commands(vec![
            "scoreboard players add base_index registers 1",
            "execute store result storage memory:temp base_index int 1.0 run scoreboard players get base_index registers",
            "data modify storage memory:stack frame append value {}",
            "",
            &format!("function {}:{}-body with storage memory:temp", dest.name(), self.ident),
            "",
            "function mcscript:pop_frame with storage memory:temp",
        ]);
        dest.append_mcfunction(entry);

        
        let mut body = Mcfunction::new(format!("{}-body", self.ident.clone()));
        self.block.to_commands(&mut body, symbol_table);

        dest.append_mcfunction(body);

        // let mut push_frame = Mcfunction::new(format!("{}-push_frame", self.ident.clone()));
        // push_frame.append_commands(vec![
            // local variables
            // "$data modify storage memory:stack frame[$(base_index)] merge value {}",
            // arguments
            // "$data modify storage memory:stack frame[$(base_index)] merge value {}",
            // base index
            // "$data modify storage memory:stack frame[$(base_index)] merge value {base_index: $(base_index)}",
        // ]);
        // dest.append_mcfunction(push_frame);
    }
}

impl Program {
    pub fn to_datapack(&self, pack_name: String) -> Datapack {
        let mut symbol_table = SymbolTable::new();

        let mut namespace = Namespace::new(pack_name.clone());
        self.func_def
            .to_mcfunction(&mut namespace, &mut symbol_table);

        let mut datapck = Datapack::new(pack_name);
        datapck.append_namespace(namespace);
        datapck
    }
}

impl Block {
    pub fn to_commands(&self, dest: &mut Mcfunction, symbol_table: &mut SymbolTable) {
        symbol_table.enter_scope();
        for block_item in &self.0 {
            match block_item {
                BlockItem::Decl(decl) => {
                    let Symbol { decorated_name } = symbol_table.new_symbol(&decl.ident);
                    let reg_res = decl.init_value.to_commands(dest, &mut 0, symbol_table);
                    dest.append_commands(vec![
                        &format!("$execute store result storage memory:stack frame[$(base_index)].{} int 1.0 run scoreboard players get {} registers", decorated_name, reg_res)
                    ]);
                }
                BlockItem::Stmt(stmt) => match stmt {
                    Stmt::Return { return_value } => {
                        let reg_res = return_value.to_commands(dest, &mut 0, symbol_table);
                        dest.append_commands(vec![&format!(
                            "scoreboard players operation return_value registers = {} registers",
                            reg_res
                        )]);
                    }
                    Stmt::Assign { ident, new_value } => {
                        let (is_local, Symbol { decorated_name }) =
                            symbol_table.query_symbol(ident);
                        let reg_res = new_value.to_commands(dest, &mut 0, symbol_table);
                        dest.append_commands(vec![
                            &if is_local {
                                format!(
                                    "$execute store result storage memory:stack frame[$(base_index)].{} int 1.0 run scoreboard players get {} registers",
                                    decorated_name, reg_res
                                )
                            } else {
                                format!(
                                    "execute store result storage memory:global {} int 1.0 run scoreboard players get {} registers",
                                    decorated_name, reg_res
                                )
                            }
                        ]);
                    }
                    Stmt::Block(block) => {
                        block.to_commands(dest, symbol_table);
                    }
                },
            }
        }
        symbol_table.leave_scope();
    }
}

impl Exp {
    pub fn to_commands(
        &self,
        dest: &mut Mcfunction,
        reg_acc: &mut u32,
        symbol_table: &mut SymbolTable,
    ) -> String {
        match self {
            Exp::Number(num) => {
                let reg_res = format!("r{}", reg_acc);
                dest.append_command(&format!(
                    "scoreboard players set {} registers {}",
                    reg_res, num
                ));
                *reg_acc += 1;
                reg_res
            }
            Exp::Variable(ident) => {
                let (is_local, Symbol { decorated_name }) = symbol_table.query_symbol(ident);
                let reg_res = format!("r{}", reg_acc);
                if is_local {
                    dest.append_command(&format!(
                        "$execute store result score {} registers run data get storage memory:stack frame[$(base_index)].{}",
                        reg_res, decorated_name
                    ));
                } else {
                    dest.append_command(&format!(
                        "$execute store result score {} registers run data get storage memory:global $({})",
                        reg_res, decorated_name
                    ));
                }
                *reg_acc += 1;
                reg_res
            }
            Exp::UnaryExp(op, exp) => {
                let reg_exp = exp.to_commands(dest, reg_acc, symbol_table);
                let reg_res = format!("r{}", reg_acc);
                *reg_acc += 1;
                match op {
                    UnaryOp::Positive => {
                        dest.append_command(&format!(
                            "scoreboard players operation {} registers = {} registers",
                            reg_res, reg_exp
                        ));
                    }
                    UnaryOp::Negative => {
                        dest.append_command(&format!(
                            "scoreboard players set {} registers 0",
                            reg_res
                        ));
                        dest.append_command(&format!(
                            "scoreboard players operation {} registers -= {} registers",
                            reg_res, reg_exp
                        ));
                    }
                }
                reg_res
            }
            Exp::BinaryExp(op, lhs, rhs) => {
                let op = match op {
                    BinaryOp::Add => "+=",
                    BinaryOp::Sub => "-=",
                    BinaryOp::Mul => "*=",
                    BinaryOp::Div => "/=",
                    BinaryOp::Mod => "%=",
                };
                let reg_lhs = lhs.to_commands(dest, reg_acc, symbol_table);
                let reg_rhs = rhs.to_commands(dest, reg_acc, symbol_table);
                let reg_res = format!("r{}", reg_acc);
                *reg_acc += 1;
                dest.append_command(&format!(
                    "scoreboard players operation {} registers = {} registers",
                    reg_res, reg_lhs
                ));
                dest.append_command(&format!(
                    "scoreboard players operation {} registers {} {} registers",
                    reg_res, op, reg_rhs
                ));
                reg_res
            }
        }
    }
}
