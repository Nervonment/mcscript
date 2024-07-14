use std::collections::HashMap;

use crate::{
    ast::{
        exp::{BinaryOp, Exp, UnaryOp},
        Block, BlockItem, FuncDef, FuncParam, Program, Stmt,
    },
    datapack::{Datapack, Mcfunction, Namespace},
};

#[derive(Clone)]
pub enum Symbol {
    Variable { decorated_name: String },
    Function { params: Vec<FuncParam> },
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

    pub fn new_variable(&mut self, ident: &str) -> Symbol {
        if self.0.last().unwrap().contains_key(ident) {
            panic!();
        }
        let mut decorated_name = ident.to_owned();
        decorated_name.push_str(&format!("@{}", self.0.len() - 1));
        let symbol = Symbol::Variable {
            decorated_name: decorated_name.clone(),
        };
        self.0
            .last_mut()
            .unwrap()
            .insert(ident.to_owned(), symbol.clone());
        symbol
    }

    pub fn new_function(&mut self, func_def: &FuncDef) -> Symbol {
        if self.0.first().unwrap().contains_key(&func_def.ident) {
            panic!();
        }
        let symbol = Symbol::Function {
            params: func_def.params.clone(),
        };
        self.0
            .first_mut()
            .unwrap()
            .insert(func_def.ident.clone(), symbol.clone());
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
    pub fn to_mcfunction(&mut self, dest: &mut Namespace, symbol_table: &mut SymbolTable) {
        symbol_table.new_function(self);

        let mut entry = Mcfunction::new(self.ident.clone());
        entry.append_commands(vec![
            "scoreboard players add base_index registers 1",
            "execute store result storage memory:temp base_index int 1.0 run scoreboard players get base_index registers",
            "data modify storage memory:stack frame append from storage memory:temp arguments",
            "",
            &format!("function {}:{}-body with storage memory:temp", dest.name(), self.ident),
            "",
            "function mcscript:pop_frame with storage memory:temp",
        ]);
        dest.append_mcfunction(entry);

        let mut branch_acc = 0;
        let mut body = Mcfunction::new(format!("{}-body", self.ident.clone()));
        symbol_table.enter_scope();
        for param in &self.params {
            symbol_table.new_variable(&param.ident);
        }
        self.block
            .to_commands(&mut body, dest, symbol_table, &mut branch_acc);
        symbol_table.leave_scope();

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
    pub fn to_datapack(&mut self, pack_name: String) -> Datapack {
        let mut symbol_table = SymbolTable::new();

        let mut namespace = Namespace::new(pack_name.clone());
        for func_def in &mut self.func_defs {
            func_def.to_mcfunction(&mut namespace, &mut symbol_table);
        }

        let mut datapck = Datapack::new(pack_name);
        datapck.append_namespace(namespace);
        datapck
    }
}

impl Block {
    pub fn to_commands(
        &mut self,
        dest: &mut Mcfunction,
        dest_namespace: &mut Namespace,
        symbol_table: &mut SymbolTable,
        branch_acc: &mut u32,
    ) {
        for block_item in &mut self.0 {
            match block_item {
                BlockItem::Decl(decl) => {
                    let decorated_name = match symbol_table.new_variable(&decl.ident) {
                        Symbol::Variable { decorated_name } => decorated_name,
                        Symbol::Function { params: _ } => unreachable!(),
                    };
                    let reg_res =
                        decl.init_value
                            .to_commands(dest, dest_namespace, &mut 0, symbol_table);
                    dest.append_commands(vec![
                        &format!("$execute store result storage memory:stack frame[$(base_index)].{} int 1.0 run scoreboard players get {} registers", decorated_name, reg_res)
                    ]);
                }
                BlockItem::Stmt(stmt) => match stmt {
                    Stmt::Return { return_value } => {
                        let reg_res =
                            return_value.to_commands(dest, dest_namespace, &mut 0, symbol_table);
                        dest.append_commands(vec![&format!(
                            "scoreboard players operation return_value registers = {} registers",
                            reg_res
                        )]);
                    }
                    Stmt::Assign { ident, new_value } => {
                        let (is_local, symbol) = symbol_table.query_symbol(ident);
                        let decorated_name = match symbol {
                            Symbol::Variable { decorated_name } => decorated_name,
                            Symbol::Function { params: _ } => panic!(),
                        };
                        let reg_res =
                            new_value.to_commands(dest, dest_namespace, &mut 0, symbol_table);
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
                        symbol_table.enter_scope();
                        block.to_commands(dest, dest_namespace, symbol_table, branch_acc);
                        symbol_table.leave_scope();
                    }
                    Stmt::IfElse {
                        exp,
                        if_branch,
                        else_branch,
                    } => {
                        let mut reg_acc = 0;
                        let reg_exp =
                            exp.to_commands(dest, dest_namespace, &mut reg_acc, symbol_table);
                        let mut if_branch_mcfuntion =
                            Mcfunction::new(format!("{}-branch_{}", dest.name(), branch_acc));
                        if else_branch.is_some() {
                            let mut else_branch_mcfuntion = Mcfunction::new(format!(
                                "{}-branch_{}",
                                dest.name(),
                                *branch_acc + 1
                            ));
                            dest.append_commands(vec![
                                &format!("scoreboard players set r{} registers 1", reg_acc),
                                &format!("execute if score {} registers matches 0 run scoreboard players set r{} registers 0", reg_exp, reg_acc),
                                &format!("execute if score r{} registers matches 1 run function {}:{} with storage memory:temp", reg_acc, dest_namespace.name(), if_branch_mcfuntion.name()),
                                &format!("execute if score {} registers matches 0 run function {}:{} with storage memory:temp", reg_exp, dest_namespace.name(), else_branch_mcfuntion.name()),
                            ]);
                            *branch_acc += 2;
                            symbol_table.enter_scope();
                            if_branch.to_commands(
                                &mut if_branch_mcfuntion,
                                dest_namespace,
                                symbol_table,
                                branch_acc,
                            );
                            symbol_table.leave_scope();
                            symbol_table.enter_scope();
                            else_branch.clone().unwrap().to_commands(
                                &mut else_branch_mcfuntion,
                                dest_namespace,
                                symbol_table,
                                branch_acc,
                            );
                            symbol_table.leave_scope();
                            dest_namespace.append_mcfunction(if_branch_mcfuntion);
                            dest_namespace.append_mcfunction(else_branch_mcfuntion);
                        } else {
                            dest.append_commands(vec![
                                &format!("scoreboard players set r{} registers 1", reg_acc),
                                &format!("execute if score {} registers matches 0 run scoreboard players set r{} registers 0", reg_exp, reg_acc),
                                &format!("execute if score r{} registers matches 1 run function {}:{} with storage memory:temp", reg_acc, dest_namespace.name(), if_branch_mcfuntion.name()),
                            ]);
                            *branch_acc += 1;
                            symbol_table.enter_scope();
                            if_branch.to_commands(
                                &mut if_branch_mcfuntion,
                                dest_namespace,
                                symbol_table,
                                branch_acc,
                            );
                            symbol_table.leave_scope();
                            dest_namespace.append_mcfunction(if_branch_mcfuntion);
                        }
                    }
                    Stmt::Exp(exp) => {
                        exp.to_commands(dest, dest_namespace, &mut 0, symbol_table);
                    }
                },
            }
        }
    }
}

impl Exp {
    pub fn to_commands(
        &mut self,
        dest: &mut Mcfunction,
        dest_namespace: &mut Namespace,
        reg_acc: &mut u32,
        symbol_table: &mut SymbolTable,
    ) -> String {
        self.call_function_first(dest, dest_namespace, reg_acc, symbol_table);
        self.to_commands_impl(dest, dest_namespace, reg_acc, symbol_table)
    }

    fn to_commands_impl(
        &self,
        dest: &mut Mcfunction,
        dest_namespace: &mut Namespace,
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
                let (is_local, symbol) = symbol_table.query_symbol(ident);
                let decorated_name = match symbol {
                    Symbol::Variable { decorated_name } => decorated_name,
                    Symbol::Function { params: _ } => panic!(),
                };
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
                let reg_exp = exp.to_commands_impl(dest, dest_namespace, reg_acc, symbol_table);
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
                let (op, is_rel, is_ne) = match op {
                    BinaryOp::Add => ("+=", false, false),
                    BinaryOp::Sub => ("-=", false, false),
                    BinaryOp::Mul => ("*=", false, false),
                    BinaryOp::Div => ("/=", false, false),
                    BinaryOp::Mod => ("%=", false, false),
                    BinaryOp::Lt => ("<", true, false),
                    BinaryOp::Le => ("<=", true, false),
                    BinaryOp::Gt => (">", true, false),
                    BinaryOp::Ge => (">=", true, false),
                    BinaryOp::Eq => ("=", true, false),
                    BinaryOp::Ne => ("=", true, true),
                };
                let reg_lhs = lhs.to_commands_impl(dest, dest_namespace, reg_acc, symbol_table);
                let reg_rhs = rhs.to_commands_impl(dest, dest_namespace, reg_acc, symbol_table);
                let reg_res = format!("r{}", reg_acc);
                if !is_rel {
                    dest.append_command(&format!(
                        "scoreboard players operation {} registers = {} registers",
                        reg_res, reg_lhs
                    ));
                    dest.append_command(&format!(
                        "scoreboard players operation {} registers {} {} registers",
                        reg_res, op, reg_rhs
                    ));
                } else if !is_ne {
                    dest.append_command(&format!("scoreboard players set {} registers 0", reg_res));
                    dest.append_command(&format!(
                        "execute if score {} registers {} {} registers run scoreboard players set {} registers 1",
                        reg_lhs, op, reg_rhs, reg_res
                    ));
                } else {
                    dest.append_command(&format!("scoreboard players set {} registers 1", reg_res));
                    dest.append_command(&format!(
                        "execute if score {} registers {} {} registers run scoreboard players set {} registers 0",
                        reg_lhs, op, reg_rhs, reg_res
                    ));
                }
                *reg_acc += 1;
                reg_res
            }
            Exp::FuncCall {
                func_ident: _,
                arguments: _,
                reg_res,
            } => reg_res.to_owned(),
        }
    }

    fn call_function_first(
        &mut self,
        dest: &mut Mcfunction,
        dest_namespace: &mut Namespace,
        reg_acc: &mut u32,
        symbol_table: &mut SymbolTable,
    ) {
        match self {
            Exp::FuncCall {
                func_ident,
                arguments,
                reg_res,
            } => {
                for (i, arg) in arguments.iter_mut().enumerate() {
                    let reg_res = arg.to_commands(dest, dest_namespace, reg_acc, symbol_table);
                    let (_, symbol) = symbol_table.query_symbol(&func_ident);
                    let params = match symbol {
                        Symbol::Variable { decorated_name: _ } => panic!(),
                        Symbol::Function { params } => params,
                    };
                    dest.append_command(
                        &format!("execute store result storage memory:temp arguments.{}@1 int 1.0 run scoreboard players get {} registers",
                        params[i].ident,
                        reg_res
                    ));
                }
                *reg_res = format!("r{}", reg_acc);
                dest.append_commands(vec![
                    &format!(
                        "function {}:{} with memory:temp",
                        dest_namespace.name(),
                        func_ident
                    ),
                    &format!(
                        "scoreboard players operation {} registers = return_value registers",
                        reg_res
                    ),
                ]);
                *reg_acc += 1;
            }
            Exp::UnaryExp(_, exp) => {
                exp.call_function_first(dest, dest_namespace, reg_acc, symbol_table);
            }
            Exp::BinaryExp(_, lhs, rhs) => {
                lhs.call_function_first(dest, dest_namespace, reg_acc, symbol_table);
                rhs.call_function_first(dest, dest_namespace, reg_acc, symbol_table);
            }
            _ => {}
        }
    }
}
