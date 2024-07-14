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

pub struct Generator {
    symbol_table: SymbolTable,
    pack_name: String,
    namespace: Namespace,
    working_function_ident: String,
    working_mcfunction: Mcfunction,
    label_acc: u32,
}

impl Generator {
    pub fn new(pack_name: String) -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            pack_name: pack_name.clone(),
            namespace: Namespace::new(pack_name),
            working_function_ident: "".into(),
            working_mcfunction: Mcfunction::new("".into()),
            label_acc: 0,
        }
    }

    pub fn generate(&mut self, mut program: Program) -> Datapack {
        for func_def in &mut program.func_defs {
            self.generate_from_func_def(func_def);
        }

        let mut datapack = Datapack::new(self.pack_name.clone());
        datapack.append_namespace(self.namespace.clone());
        datapack
    }

    fn generate_from_func_def(&mut self, func_def: &mut FuncDef) {
        self.working_function_ident = func_def.ident.clone();
        self.symbol_table.new_function(func_def);

        let mut entry = Mcfunction::new(self.working_function_ident.clone());
        entry.append_commands(vec![
            "scoreboard players add base_index registers 1",
            "execute store result storage memory:temp base_index int 1.0 run scoreboard players get base_index registers",
            "data modify storage memory:stack frame append from storage memory:temp arguments",
            "",
            &format!("function {}:{}-label_0 with storage memory:temp", self.namespace.name(), self.working_function_ident.clone()),
            "",
            "function mcscript:pop_frame with storage memory:temp",
        ]);
        self.namespace.append_mcfunction(entry);

        self.label_acc = 0;
        self.working_mcfunction =
            Mcfunction::new(format!("{}-label_0", self.working_function_ident));
        self.symbol_table.enter_scope();
        for param in &func_def.params {
            self.symbol_table.new_variable(&param.ident);
        }
        self.generate_from_block(&mut func_def.block);
        self.symbol_table.leave_scope();

        self.namespace
            .append_mcfunction(self.working_mcfunction.clone());
    }

    fn generate_from_block(&mut self, block: &mut Block) {
        for block_item in &mut block.0 {
            match block_item {
                BlockItem::Decl(decl) => {
                    let decorated_name = match self.symbol_table.new_variable(&decl.ident) {
                        Symbol::Variable { decorated_name } => decorated_name,
                        Symbol::Function { params: _ } => unreachable!(),
                    };
                    let reg_res = self.generate_from_exp(&mut decl.init_value, &mut 0);
                    self.working_mcfunction.append_commands(vec![
                        &format!("$execute store result storage memory:stack frame[$(base_index)].{} int 1.0 run scoreboard players get {} registers", decorated_name, reg_res)
                    ]);
                }
                BlockItem::Stmt(stmt) => match stmt {
                    Stmt::Return { return_value } => {
                        let reg_res = self.generate_from_exp(return_value, &mut 0);
                        self.working_mcfunction.append_commands(vec![
                            &format!(
                                "scoreboard players operation return_value registers = {} registers",
                                reg_res
                            ),
                            // "function mcscript:pop_frame with storage memory:temp",
                            "return 0",
                        ]);
                    }
                    Stmt::Assign { ident, new_value } => {
                        let (is_local, symbol) = self.symbol_table.query_symbol(ident);
                        let decorated_name = match symbol {
                            Symbol::Variable { decorated_name } => decorated_name,
                            Symbol::Function { params: _ } => panic!(),
                        };
                        let reg_res = self.generate_from_exp(new_value, &mut 0);
                        self.working_mcfunction.append_commands(vec![
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
                        self.symbol_table.enter_scope();
                        self.generate_from_block(block);
                        self.symbol_table.leave_scope();
                    }
                    Stmt::IfElse {
                        exp,
                        if_branch,
                        else_branch,
                    } => {
                        let mut reg_acc = 0;
                        let reg_exp = self.generate_from_exp(exp, &mut reg_acc);

                        let label_if_branch = format!(
                            "{}-label_{}",
                            self.working_function_ident,
                            self.label_acc + 1
                        );
                        if else_branch.is_some() {
                            let label_else_branch = format!(
                                "{}-label_{}",
                                self.working_function_ident,
                                self.label_acc + 2
                            );
                            let label_following = format!(
                                "{}-label_{}",
                                self.working_function_ident,
                                self.label_acc + 3
                            );
                            self.working_mcfunction.append_commands(vec![
                                &format!("scoreboard players set r{} registers 1", reg_acc),
                                &format!("execute if score {} registers matches 0 run scoreboard players set r{} registers 0", reg_exp, reg_acc),
                                &format!("execute if score r{} registers matches 1 run function {}:{} with storage memory:temp", reg_acc, self.namespace.name(), label_if_branch),
                                &format!("execute if score {} registers matches 0 run function {}:{} with storage memory:temp", reg_exp, self.namespace.name(), label_else_branch),
                            ]);
                            // if branch
                            self.new_label();
                            self.generate_from_block(if_branch);
                            self.working_mcfunction.append_command(&format!(
                                "function {}:{} with storage memory:temp",
                                self.namespace.name(),
                                label_following
                            ));
                            // else branch
                            self.new_label();
                            self.generate_from_block(&mut else_branch.clone().unwrap());
                            self.working_mcfunction.append_command(&format!(
                                "function {}:{} with storage memory:temp",
                                self.namespace.name(),
                                label_following
                            ));
                            // following
                            self.new_label();
                        } else {
                            let label_following = format!(
                                "{}-label_{}",
                                self.working_function_ident,
                                self.label_acc + 2
                            );
                            self.working_mcfunction.append_commands(vec![
                                &format!("scoreboard players set r{} registers 1", reg_acc),
                                &format!("execute if score {} registers matches 0 run scoreboard players set r{} registers 0", reg_exp, reg_acc),
                                &format!("execute if score r{} registers matches 1 run function {}:{} with storage memory:temp", reg_acc, self.namespace.name(), label_if_branch),
                                &format!("execute if score {} registers matches 0 run function {}:{} with storage memory:temp", reg_exp, self.namespace.name(), label_following)
                            ]);
                            // if branch
                            self.new_label();
                            self.generate_from_block(if_branch);
                            self.working_mcfunction.append_command(&format!(
                                "function {}:{} with storage memory:temp",
                                self.namespace.name(),
                                label_following
                            ));
                            // following
                            self.new_label();
                        }
                    }
                    Stmt::Exp(exp) => {
                        self.generate_from_exp(exp, &mut 0);
                    }
                },
            }
        }
    }

    fn generate_from_exp(&mut self, exp: &mut Exp, reg_acc: &mut u32) -> String {
        self.generate_from_exp_call_function_first(exp, reg_acc);
        self.generate_from_exp_eval(exp, reg_acc)
    }

    fn generate_from_exp_eval(&mut self, exp: &mut Exp, reg_acc: &mut u32) -> String {
        match exp {
            Exp::Number(num) => {
                let reg_res = format!("r{}", reg_acc);
                self.working_mcfunction.append_command(&format!(
                    "scoreboard players set {} registers {}",
                    reg_res, num
                ));
                *reg_acc += 1;
                reg_res
            }
            Exp::Variable(ident) => {
                let (is_local, symbol) = self.symbol_table.query_symbol(ident);
                let decorated_name = match symbol {
                    Symbol::Variable { decorated_name } => decorated_name,
                    Symbol::Function { params: _ } => panic!(),
                };
                let reg_res = format!("r{}", reg_acc);
                if is_local {
                    self.working_mcfunction.append_command(&format!(
                        "$execute store result score {} registers run data get storage memory:stack frame[$(base_index)].{}",
                        reg_res, decorated_name
                    ));
                } else {
                    self.working_mcfunction.append_command(&format!(
                        "$execute store result score {} registers run data get storage memory:global $({})",
                        reg_res, decorated_name
                    ));
                }
                *reg_acc += 1;
                reg_res
            }
            Exp::UnaryExp(op, exp) => {
                let reg_exp = self.generate_from_exp_eval(exp, reg_acc);
                let reg_res = format!("r{}", reg_acc);
                *reg_acc += 1;
                match op {
                    UnaryOp::Positive => {
                        self.working_mcfunction.append_command(&format!(
                            "scoreboard players operation {} registers = {} registers",
                            reg_res, reg_exp
                        ));
                    }
                    UnaryOp::Negative => {
                        self.working_mcfunction.append_command(&format!(
                            "scoreboard players set {} registers 0",
                            reg_res
                        ));
                        self.working_mcfunction.append_command(&format!(
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
                let reg_lhs = self.generate_from_exp_eval(lhs, reg_acc);
                let reg_rhs = self.generate_from_exp_eval(rhs, reg_acc);
                let reg_res = format!("r{}", reg_acc);
                if !is_rel {
                    self.working_mcfunction.append_command(&format!(
                        "scoreboard players operation {} registers = {} registers",
                        reg_res, reg_lhs
                    ));
                    self.working_mcfunction.append_command(&format!(
                        "scoreboard players operation {} registers {} {} registers",
                        reg_res, op, reg_rhs
                    ));
                } else if !is_ne {
                    self.working_mcfunction
                        .append_command(&format!("scoreboard players set {} registers 0", reg_res));
                    self.working_mcfunction.append_command(&format!(
                        "execute if score {} registers {} {} registers run scoreboard players set {} registers 1",
                        reg_lhs, op, reg_rhs, reg_res
                    ));
                } else {
                    self.working_mcfunction
                        .append_command(&format!("scoreboard players set {} registers 1", reg_res));
                    self.working_mcfunction.append_command(&format!(
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

    fn generate_from_exp_call_function_first(&mut self, exp: &mut Exp, reg_acc: &mut u32) {
        match exp {
            Exp::FuncCall {
                func_ident,
                arguments,
                reg_res,
            } => {
                for (i, arg) in arguments.iter_mut().enumerate() {
                    let reg_res = self.generate_from_exp(arg, reg_acc);
                    let (_, symbol) = self.symbol_table.query_symbol(&func_ident);
                    let params = match symbol {
                        Symbol::Variable { decorated_name: _ } => panic!(),
                        Symbol::Function { params } => params,
                    };
                    self.working_mcfunction.append_command(
                        &format!("execute store result storage memory:temp arguments.{}@1 int 1.0 run scoreboard players get {} registers",
                        params[i].ident,
                        reg_res
                    ));
                }
                *reg_res = format!("r{}", reg_acc);
                self.working_mcfunction.append_commands(vec![
                    &format!(
                        "function {}:{} with storage memory:temp",
                        self.namespace.name(),
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
                self.generate_from_exp_call_function_first(exp, reg_acc);
            }
            Exp::BinaryExp(_, lhs, rhs) => {
                self.generate_from_exp_call_function_first(lhs, reg_acc);
                self.generate_from_exp_call_function_first(rhs, reg_acc);
            }
            _ => {}
        }
    }

    fn new_label(&mut self) -> String {
        self.namespace
            .append_mcfunction(self.working_mcfunction.clone());
        self.label_acc += 1;
        let mcfunction_name = format!("{}-label_{}", self.working_function_ident, self.label_acc);
        self.working_mcfunction = Mcfunction::new(mcfunction_name.clone());
        mcfunction_name
    }
}
