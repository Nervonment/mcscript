use std::collections::HashMap;

use crate::{
    ast::{
        exp::{BinaryOp, Exp, UnaryOp},
        Block, BlockItem, DataType, FuncDef, FuncParam, Program, Stmt,
    },
    datapack::{Mcfunction, Namespace},
};

#[derive(Clone)]
struct Symbol {
    decorated_name: String,
}

struct SymbolTable(Vec<HashMap<String, Symbol>>);

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
        let symbol = Symbol {
            decorated_name: decorated_name.clone(),
        };
        self.0
            .last_mut()
            .unwrap()
            .insert(ident.to_owned(), symbol.clone());
        symbol
    }

    pub fn set_parameters(&mut self, params: &Vec<FuncParam>) {
        for (i, param) in params.iter().enumerate() {
            let decorated_name = format!("%{}", i);
            self.0
                .last_mut()
                .unwrap()
                .insert(param.ident.clone(), Symbol { decorated_name });
        }
    }

    // pub fn new_function(&mut self, func_def: &FuncDef) -> Symbol {
    //     if self.0.first().unwrap().contains_key(&func_def.ident) {
    //         panic!();
    //     }
    //     let symbol = Symbol::Function {
    //         params: func_def.params.clone(),
    //     };
    //     self.0
    //         .first_mut()
    //         .unwrap()
    //         .insert(func_def.ident.clone(), symbol.clone());
    //     symbol
    // }

    pub fn query_symbol(&self, ident: &str) -> (bool, Symbol) {
        for (level, scope) in self.0.iter().enumerate().rev() {
            if scope.contains_key(ident) {
                return (level > 0, scope[ident].clone());
            }
        }
        panic!();
    }
}

struct Path(String, String);

enum ExpVal {
    Int { reg: String },
    Array { element_type: DataType, path: Path },
}

pub struct Generator {
    symbol_table: SymbolTable,
    namespace: Namespace,
    working_function_ident: String,
    working_mcfunction: Option<Mcfunction>,
    label_acc: u32,
    break_labels: Vec<String>,
    continue_labels: Vec<String>,
}

impl Generator {
    pub fn new(namespace: String) -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            namespace: Namespace::new(namespace),
            working_function_ident: "".into(),
            working_mcfunction: None,
            label_acc: 0,
            break_labels: vec![],
            continue_labels: vec![],
        }
    }

    pub fn generate(&mut self, mut program: Program) -> Namespace {
        for func_def in &mut program.func_defs {
            self.generate_from_func_def(func_def);
        }

        self.namespace.clone()
    }

    fn generate_from_func_def(&mut self, func_def: &mut FuncDef) {
        self.working_function_ident = func_def.ident.clone();
        // self.symbol_table.new_function(func_def);

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
        self.working_mcfunction = Some(self.new_label());
        self.symbol_table.enter_scope();
        self.symbol_table.set_parameters(&func_def.params);
        // for param in &func_def.params {
        //     self.symbol_table.new_variable(&param.ident);
        // }
        self.generate_from_block(&mut func_def.block);
        self.symbol_table.leave_scope();

        self.namespace
            .append_mcfunction(self.working_mcfunction.take().unwrap());
    }

    fn generate_from_block(&mut self, block: &mut Block) {
        for block_item in &mut block.0 {
            match block_item {
                BlockItem::Decl(decl) => {
                    // let decorated_name = match self.symbol_table.new_variable(&decl.ident) {
                    //     Symbol::Variable { decorated_name } => decorated_name,
                    //     Symbol::Function { params: _ } => unreachable!(),
                    // };
                    let decorated_name = self.symbol_table.new_variable(&decl.ident).decorated_name;
                    let reg_res = self.generate_from_exp(&mut decl.init_value, &mut 0);
                    self.working_mcfunction.as_mut().unwrap().append_commands(vec![
                        &format!("$execute store result storage memory:stack frame[$(base_index)].{} int 1.0 run scoreboard players get {} registers", decorated_name, reg_res)
                    ]);
                }
                BlockItem::Stmt(stmt) => match stmt {
                    Stmt::Return { return_value } => {
                        let reg_res = self.generate_from_exp(return_value, &mut 0);
                        self.working_mcfunction.as_mut().unwrap().append_commands(vec![
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
                        // let decorated_name = match symbol {
                        //     Symbol::Variable { decorated_name } => decorated_name,
                        //     Symbol::Function { params: _ } => panic!(),
                        // };
                        let decorated_name = symbol.decorated_name;
                        let reg_res = self.generate_from_exp(new_value, &mut 0);
                        self.working_mcfunction.as_mut().unwrap().append_commands(vec![
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
                        let reg_exp = self.generate_from_exp(exp, &mut 0);

                        let label_if_branch = self.new_label();
                        if else_branch.is_some() {
                            let label_else_branch = self.new_label();
                            let label_following = self.new_label();
                            self.working_mcfunction.as_mut().unwrap().append_commands(vec![
                                &format!("execute if score {} registers matches 0 run return run function {}:{} with storage memory:temp", reg_exp, self.namespace.name(), label_else_branch.name()),
                                &format!("function {}:{} with storage memory:temp", self.namespace.name(), label_if_branch.name()),
                            ]);
                            // if branch
                            self.work_with_next_mcfunction(label_if_branch);
                            self.symbol_table.enter_scope();
                            self.generate_from_block(if_branch);
                            self.symbol_table.leave_scope();
                            self.working_mcfunction
                                .as_mut()
                                .unwrap()
                                .append_command(&format!(
                                    "function {}:{} with storage memory:temp",
                                    self.namespace.name(),
                                    label_following.name()
                                ));
                            // else branch
                            self.work_with_next_mcfunction(label_else_branch);
                            self.symbol_table.enter_scope();
                            self.generate_from_block(&mut else_branch.clone().unwrap());
                            self.symbol_table.leave_scope();
                            self.working_mcfunction
                                .as_mut()
                                .unwrap()
                                .append_command(&format!(
                                    "function {}:{} with storage memory:temp",
                                    self.namespace.name(),
                                    label_following.name()
                                ));
                            // following
                            self.work_with_next_mcfunction(label_following);
                        } else {
                            let label_following = self.new_label();
                            self.working_mcfunction.as_mut().unwrap().append_commands(vec![
                                &format!("execute if score {} registers matches 0 run return run function {}:{} with storage memory:temp", reg_exp, self.namespace.name(), label_following.name()),
                                &format!("function {}:{} with storage memory:temp", self.namespace.name(), label_if_branch.name()),
                            ]);
                            // if branch
                            self.work_with_next_mcfunction(label_if_branch);
                            self.symbol_table.enter_scope();
                            self.generate_from_block(if_branch);
                            self.symbol_table.leave_scope();
                            self.working_mcfunction
                                .as_mut()
                                .unwrap()
                                .append_command(&format!(
                                    "function {}:{} with storage memory:temp",
                                    self.namespace.name(),
                                    label_following.name()
                                ));
                            // following
                            self.work_with_next_mcfunction(label_following);
                        }
                    }
                    Stmt::While { exp, body } => {
                        let label_judge = self.new_label();
                        let label_while_body = self.new_label();
                        let label_following = self.new_label();

                        let label_judge_name = label_judge.name().to_owned();

                        self.break_labels.push(label_following.name().to_owned());
                        self.continue_labels.push(label_judge.name().to_owned());

                        self.working_mcfunction
                            .as_mut()
                            .unwrap()
                            .append_commands(vec![&format!(
                                "function {}:{} with storage memory:temp",
                                self.namespace.name(),
                                label_judge.name()
                            )]);
                        // judge
                        self.work_with_next_mcfunction(label_judge);
                        let reg_exp = self.generate_from_exp(exp, &mut 0);
                        self.working_mcfunction.as_mut().unwrap().append_commands(vec![
                            &format!("execute if score {} registers matches 0 run return run function {}:{} with storage memory:temp", reg_exp, self.namespace.name(), label_following.name()),
                            &format!("function {}:{} with storage memory:temp", self.namespace.name(), label_while_body.name()),
                        ]);
                        // while body
                        self.work_with_next_mcfunction(label_while_body);
                        self.symbol_table.enter_scope();
                        self.generate_from_block(body);
                        self.symbol_table.leave_scope();
                        self.working_mcfunction
                            .as_mut()
                            .unwrap()
                            .append_commands(vec![
                                "",
                                &format!(
                                    "function {}:{} with storage memory:temp",
                                    self.namespace.name(),
                                    label_judge_name
                                ),
                            ]);
                        // following
                        self.break_labels.pop();
                        self.continue_labels.pop();
                        self.work_with_next_mcfunction(label_following);
                    }
                    Stmt::Exp(exp) => {
                        self.generate_from_exp(exp, &mut 0);
                    }
                    Stmt::Break => {
                        self.working_mcfunction
                            .as_mut()
                            .unwrap()
                            .append_command(&format!(
                                "return run function {}:{} with storage memory:temp",
                                self.namespace.name(),
                                self.break_labels.last().unwrap(),
                            ));
                    }
                    Stmt::Continue => {
                        self.working_mcfunction
                            .as_mut()
                            .unwrap()
                            .append_command(&format!(
                                "return run function {}:{} with storage memory:temp",
                                self.namespace.name(),
                                self.continue_labels.last().unwrap(),
                            ));
                    }
                },
            }
        }
    }

    fn generate_from_exp(&mut self, exp: &mut Exp, reg_acc: &mut u32) -> String {
        match exp {
            Exp::Number(num) => {
                let reg_res = format!("r{}", reg_acc);
                self.working_mcfunction
                    .as_mut()
                    .unwrap()
                    .append_command(&format!(
                        "scoreboard players set {} registers {}",
                        reg_res, num
                    ));
                *reg_acc += 1;
                reg_res
            }
            Exp::Variable(ident) => {
                let (is_local, symbol) = self.symbol_table.query_symbol(ident);
                // let decorated_name = match symbol {
                //     Symbol::Variable { decorated_name } => decorated_name,
                //     Symbol::Function { params: _ } => panic!(),
                // };
                let decorated_name = symbol.decorated_name;
                let reg_res = format!("r{}", reg_acc);
                if is_local {
                    self.working_mcfunction.as_mut().unwrap().append_command(&format!(
                        "$execute store result score {} registers run data get storage memory:stack frame[$(base_index)].{}",
                        reg_res, decorated_name
                    ));
                } else {
                    self.working_mcfunction.as_mut().unwrap().append_command(&format!(
                        "$execute store result score {} registers run data get storage memory:global $({})",
                        reg_res, decorated_name
                    ));
                }
                *reg_acc += 1;
                reg_res
            }
            Exp::UnaryExp(op, exp) => {
                let reg_exp = self.generate_from_exp(exp, reg_acc);
                let reg_res = format!("r{}", reg_acc);
                *reg_acc += 1;
                match op {
                    UnaryOp::Positive => {
                        self.working_mcfunction
                            .as_mut()
                            .unwrap()
                            .append_command(&format!(
                                "scoreboard players operation {} registers = {} registers",
                                reg_res, reg_exp
                            ));
                    }
                    UnaryOp::Negative => {
                        self.working_mcfunction
                            .as_mut()
                            .unwrap()
                            .append_command(&format!(
                                "scoreboard players set {} registers 0",
                                reg_res
                            ));
                        self.working_mcfunction
                            .as_mut()
                            .unwrap()
                            .append_command(&format!(
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
                let reg_lhs = self.generate_from_exp(lhs, reg_acc);
                let reg_rhs = self.generate_from_exp(rhs, reg_acc);
                let reg_res = format!("r{}", reg_acc);
                if !is_rel {
                    self.working_mcfunction
                        .as_mut()
                        .unwrap()
                        .append_command(&format!(
                            "scoreboard players operation {} registers = {} registers",
                            reg_res, reg_lhs
                        ));
                    self.working_mcfunction
                        .as_mut()
                        .unwrap()
                        .append_command(&format!(
                            "scoreboard players operation {} registers {} {} registers",
                            reg_res, op, reg_rhs
                        ));
                } else if !is_ne {
                    self.working_mcfunction
                        .as_mut()
                        .unwrap()
                        .append_command(&format!("scoreboard players set {} registers 0", reg_res));
                    self.working_mcfunction.as_mut().unwrap().append_command(&format!(
                        "execute if score {} registers {} {} registers run scoreboard players set {} registers 1",
                        reg_lhs, op, reg_rhs, reg_res
                    ));
                } else {
                    self.working_mcfunction
                        .as_mut()
                        .unwrap()
                        .append_command(&format!("scoreboard players set {} registers 1", reg_res));
                    self.working_mcfunction.as_mut().unwrap().append_command(&format!(
                        "execute if score {} registers {} {} registers run scoreboard players set {} registers 0",
                        reg_lhs, op, reg_rhs, reg_res
                    ));
                }
                *reg_acc += 1;
                reg_res
            }
            Exp::FuncCall {
                namespace,
                func_ident,
                arguments,
            } => {
                let reg_res = format!("r{}", reg_acc);
                // save registers
                for i in 0..*reg_acc {
                    self.working_mcfunction.as_mut().unwrap().append_command(
                        &format!("$execute store result storage memory:stack frame[$(base_index)].%r{} int 1.0 run scoreboard players get r{} registers", i, i)
                    );
                }
                // call function
                let mut reg_acc_1 = 0;
                for (i, arg) in arguments.iter_mut().enumerate() {
                    let reg_res = self.generate_from_exp(arg, &mut reg_acc_1);
                    // let (_, symbol) = self.symbol_table.query_symbol(&func_ident);
                    // let params = match symbol {
                    //     Symbol::Variable { decorated_name: _ } => panic!(),
                    //     Symbol::Function { params } => params,
                    // };
                    self.working_mcfunction.as_mut().unwrap().append_command(
                        &format!("execute store result storage memory:temp arguments.%{} int 1.0 run scoreboard players get {} registers",
                        i,
                        reg_res
                    ));
                }
                self.working_mcfunction
                    .as_mut()
                    .unwrap()
                    .append_commands(vec![
                        &format!(
                            "function {}:{} with storage memory:temp",
                            if namespace.is_some() {
                                namespace.clone().unwrap()
                            } else {
                                self.namespace.name().to_owned()
                            },
                            func_ident
                        ),
                        &format!(
                            "scoreboard players operation {} registers = return_value registers",
                            reg_res
                        ),
                    ]);
                // restore registers
                for i in 0..*reg_acc {
                    self.working_mcfunction.as_mut().unwrap().append_command(
                        &format!("$execute store result score r{} registers run data get storage memory:stack frame[$(base_index)].%r{}", i, i)
                    );
                }
                *reg_acc += 1;
                reg_res
            }
        }
    }

    fn new_label(&mut self) -> Mcfunction {
        let mcfunction_name = format!("{}-label_{}", self.working_function_ident, self.label_acc);
        self.label_acc += 1;
        Mcfunction::new(mcfunction_name)
    }

    fn work_with_next_mcfunction(&mut self, next_mcfunction: Mcfunction) {
        self.namespace
            .append_mcfunction(self.working_mcfunction.take().unwrap());
        self.working_mcfunction = Some(next_mcfunction);
    }
}
