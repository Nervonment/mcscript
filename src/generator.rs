use std::collections::HashMap;

use crate::{
    ast::{
        exp::{BinaryOp, Exp, UnaryOp},
        Block, BlockItem, CompileUnit, DataType, FuncDef, FuncParam, Stmt,
    },
    datapack::{Datapack, Mcfunction, Namespace},
};

#[derive(Clone)]
struct Variable {
    decorated_name: String,
    data_type: DataType,
}

struct VariableTable(Vec<HashMap<String, Variable>>);

impl VariableTable {
    pub fn new() -> Self {
        Self(vec![HashMap::<String, Variable>::new()])
    }

    pub fn enter_scope(&mut self) {
        self.0.push(HashMap::<_, _>::new());
    }

    pub fn leave_scope(&mut self) {
        self.0.pop();
    }

    pub fn new_variable(&mut self, ident: &str, data_type: DataType) -> Variable {
        if self.0.last().unwrap().contains_key(ident) {
            panic!();
        }
        let mut decorated_name = ident.to_owned();
        decorated_name.push_str(&format!("@{}", self.0.len() - 1));
        let variable = Variable {
            decorated_name: decorated_name.clone(),
            data_type,
        };
        self.0
            .last_mut()
            .unwrap()
            .insert(ident.to_owned(), variable.clone());
        variable
    }

    pub fn set_parameters(&mut self, params: &Vec<FuncParam>) {
        for (i, param) in params.iter().enumerate() {
            let decorated_name = format!("%{}", i);
            self.0.last_mut().unwrap().insert(
                param.ident.clone(),
                Variable {
                    decorated_name,
                    data_type: param.data_type.clone(),
                },
            );
        }
    }

    pub fn query_variable(&self, ident: &str) -> (bool, Variable) {
        for (level, scope) in self.0.iter().enumerate().rev() {
            if scope.contains_key(ident) {
                return (level > 0, scope[ident].clone());
            }
        }
        panic!();
    }
}

struct FunctionTable(HashMap<(String, String), FuncDef>);

impl FunctionTable {
    pub fn new() -> Self {
        Self(HashMap::<_, _>::new())
    }

    pub fn new_function(&mut self, key: (String, String), func_def: FuncDef) {
        if self.0.contains_key(&key) {
            panic!();
        }
        self.0.insert(key, func_def);
    }

    pub fn query_function(&self, key: (&str, &str)) -> &FuncDef {
        self.0.get(&(key.0.to_owned(), key.1.to_owned())).unwrap()
    }
}

struct Path(String, String);

enum ExpVal {
    Int {
        reg: String,
    },
    Array {
        element_type: DataType,
        path_path: Path,
    },
}

pub struct Generator {
    variable_table: VariableTable,
    function_table: FunctionTable,
    datapack: Datapack,
    working_namespace: Option<Namespace>,
    working_function_ident: String,
    working_mcfunction: Option<Mcfunction>,
    label_acc: u32,
    break_labels: Vec<String>,
    continue_labels: Vec<String>,
}

impl Generator {
    pub fn new(pack_name: String) -> Self {
        Self {
            variable_table: VariableTable::new(),
            function_table: FunctionTable::new(),
            datapack: Datapack::new(pack_name),
            working_namespace: None,
            working_function_ident: "".into(),
            working_mcfunction: None,
            label_acc: 0,
            break_labels: vec![],
            continue_labels: vec![],
        }
    }

    pub fn generate(&mut self, compile_units: Vec<(CompileUnit, String)>) -> Datapack {
        for (compile_unit, namespace) in &compile_units {
            self.scan_func_defs(&compile_unit, &namespace);
        }
        for (compile_unit, namespace) in compile_units {
            self.generate_from_namespace(compile_unit, namespace);
        }
        self.datapack.clone()
    }

    fn scan_func_defs(&mut self, compile_unit: &CompileUnit, namespace: &str) {
        for func_def in &compile_unit.func_defs {
            self.function_table.new_function(
                (namespace.to_owned(), func_def.ident.clone()),
                func_def.clone(),
            );
        }
    }

    fn generate_from_namespace(&mut self, mut compile_unit: CompileUnit, namespace: String) {
        self.working_namespace = Some(Namespace::new(namespace));

        for func_def in &mut compile_unit.func_defs {
            self.generate_from_func_def(func_def);
        }

        self.datapack
            .append_namespace(self.working_namespace.take().unwrap());
    }

    fn generate_from_func_def(&mut self, func_def: &mut FuncDef) {
        self.working_function_ident = func_def.ident.clone();

        let mut entry = Mcfunction::new(self.working_function_ident.clone());
        entry.append_commands(vec![
            "scoreboard players add base_index registers 1",
            "execute store result storage memory:temp base_index int 1.0 run scoreboard players get base_index registers",
            "data modify storage memory:stack frame append from storage memory:temp arguments",
            "",
            &format!("function {}:{}-label_0 with storage memory:temp", self.working_namespace.as_ref().unwrap().name(), self.working_function_ident.clone()),
            "",
            "function mcscript:pop_frame with storage memory:temp",
        ]);
        self.working_namespace
            .as_mut()
            .unwrap()
            .append_mcfunction(entry);

        self.label_acc = 0;
        self.working_mcfunction = Some(self.new_label());
        self.variable_table.enter_scope();
        self.variable_table.set_parameters(&func_def.params);
        self.generate_from_block(&mut func_def.block);
        self.variable_table.leave_scope();

        self.working_namespace
            .as_mut()
            .unwrap()
            .append_mcfunction(self.working_mcfunction.take().unwrap());
    }

    fn generate_from_block(&mut self, block: &mut Block) {
        for block_item in &mut block.0 {
            match block_item {
                BlockItem::Decl(decl) => {
                    let exp_val =
                        self.generate_from_exp(&mut decl.init_value, &mut 0, &mut 0, &mut 0);
                    match exp_val {
                        ExpVal::Int { reg } => {
                            let decorated_name = self
                                .variable_table
                                .new_variable(&decl.ident, DataType::Int)
                                .decorated_name;
                            self.working_mcfunction.as_mut().unwrap().append_commands(vec![
                                &format!("$execute store result storage memory:stack frame[$(base_index)].{} int 1.0 run scoreboard players get {} registers", decorated_name, reg)
                            ]);
                        }
                        ExpVal::Array {
                            element_type,
                            path_path,
                        } => {
                            let decorated_name = self
                                .variable_table
                                .new_variable(
                                    &decl.ident,
                                    DataType::Array {
                                        element_type: Box::new(element_type),
                                    },
                                )
                                .decorated_name;
                            self.working_mcfunction.as_mut().unwrap().append_commands(vec![
                                &format!("$data modify storage memory:temp target_path set value \"memory:stack frame[$(base_index)].{}\"", decorated_name),
                                &format!("$data modify storage memory:temp src_path set from storage {} {}", path_path.0, path_path.1),
                                &format!("function mcscript:mov_m_m with storage memory:temp"),
                            ]);
                        }
                    }
                }
                BlockItem::Stmt(stmt) => match stmt {
                    Stmt::Return { return_value } => {
                        let func_def = self
                            .function_table
                            .query_function((
                                self.working_namespace.as_ref().unwrap().name(),
                                &self.working_function_ident,
                            ))
                            .clone();
                        if return_value.is_some() {
                            let return_value = return_value.as_mut().unwrap();
                            let exp_val =
                                self.generate_from_exp(return_value, &mut 0, &mut 0, &mut 0);

                            match exp_val {
                                ExpVal::Int { reg } => {
                                    if *func_def.func_type.as_ref().unwrap() != DataType::Int {
                                        panic!();
                                    }
                                    self.working_mcfunction.as_mut().unwrap().append_commands(vec![
                                    &format!(
                                        "scoreboard players operation return_value registers = {} registers",
                                        reg
                                    ),
                                    "return 0",
                                ]);
                                }
                                ExpVal::Array {
                                    element_type,
                                    path_path,
                                } => match func_def.func_type.as_ref().unwrap() {
                                    DataType::Array {
                                        element_type: element_type_need,
                                    } => {
                                        if element_type != *element_type_need.as_ref() {
                                            panic!();
                                        }
                                        self.working_mcfunction.as_mut().unwrap().append_commands(vec![
                                            "data modify storage memory:temp target_path set value \"memory:temp return_array\"",
                                            &format!("$data modify storage memory:temp src_path set from storage {} {}", path_path.0, path_path.1),
                                            "function mcscript:mov_m_m with storage memory:temp",
                                            "return 0",
                                        ]);
                                    }
                                    _ => unreachable!(),
                                },
                            }
                        } else {
                            if func_def.func_type.is_some() {
                                panic!();
                            } else {
                                self.working_mcfunction
                                    .as_mut()
                                    .unwrap()
                                    .append_command("return 0");
                            }
                        }
                    }
                    Stmt::Assign { ident, new_value } => {
                        let (is_local, variable) = self.variable_table.query_variable(ident);
                        let decorated_name = variable.decorated_name;
                        let exp_val = self.generate_from_exp(new_value, &mut 0, &mut 0, &mut 0);
                        match exp_val {
                            ExpVal::Int { reg } => match variable.data_type {
                                DataType::Int => {
                                    self.working_mcfunction.as_mut().unwrap().append_commands(vec![
                                        &if is_local {
                                            format!(
                                                "$execute store result storage memory:stack frame[$(base_index)].{} int 1.0 run scoreboard players get {} registers",
                                                decorated_name, reg
                                            )
                                        } else {
                                            panic!();
                                        }
                                    ]);
                                }
                                _ => unreachable!(),
                            },
                            ExpVal::Array {
                                element_type,
                                path_path,
                            } => match variable.data_type {
                                DataType::Array {
                                    element_type: recv_element_type,
                                } => {
                                    if *recv_element_type != element_type {
                                        panic!();
                                    }
                                    if is_local {
                                        self.working_mcfunction
                                            .as_mut()
                                            .unwrap()
                                            .append_commands(vec![
                                                &format!("$data modify memory:temp target_path set value \"memory:stack frame[$(base_index)].{}\"", decorated_name),
                                                &format!("$data modify memory:temp src_path set from storage {} {}", path_path.0, path_path.1),
                                                "function mcscript:mov_m_m with storage memory:temp"
                                            ]);
                                    } else {
                                        panic!();
                                    }
                                }
                                _ => unreachable!(),
                            },
                        }
                    }
                    Stmt::Block(block) => {
                        self.variable_table.enter_scope();
                        self.generate_from_block(block);
                        self.variable_table.leave_scope();
                    }
                    Stmt::IfElse {
                        exp,
                        if_branch,
                        else_branch,
                    } => {
                        let exp_val = self.generate_from_exp(exp, &mut 0, &mut 0, &mut 0);

                        match exp_val {
                            ExpVal::Int { reg } => {
                                let label_if_branch = self.new_label();
                                if else_branch.is_some() {
                                    let label_else_branch = self.new_label();
                                    let label_following = self.new_label();
                                    self.working_mcfunction.as_mut().unwrap().append_commands(vec![
                                        &format!("execute if score {} registers matches 0 run return run function {}:{} with storage memory:temp", reg, self.working_namespace.as_ref().unwrap().name(), label_else_branch.name()),
                                        &format!("function {}:{} with storage memory:temp", self.working_namespace.as_ref().unwrap().name(), label_if_branch.name()),
                                    ]);
                                    // if branch
                                    self.work_with_next_mcfunction(label_if_branch);
                                    self.variable_table.enter_scope();
                                    self.generate_from_block(if_branch);
                                    self.variable_table.leave_scope();
                                    self.working_mcfunction.as_mut().unwrap().append_command(
                                        &format!(
                                            "function {}:{} with storage memory:temp",
                                            self.working_namespace.as_ref().unwrap().name(),
                                            label_following.name()
                                        ),
                                    );
                                    // else branch
                                    self.work_with_next_mcfunction(label_else_branch);
                                    self.variable_table.enter_scope();
                                    self.generate_from_block(&mut else_branch.clone().unwrap());
                                    self.variable_table.leave_scope();
                                    self.working_mcfunction.as_mut().unwrap().append_command(
                                        &format!(
                                            "function {}:{} with storage memory:temp",
                                            self.working_namespace.as_ref().unwrap().name(),
                                            label_following.name()
                                        ),
                                    );
                                    // following
                                    self.work_with_next_mcfunction(label_following);
                                } else {
                                    let label_following = self.new_label();
                                    self.working_mcfunction.as_mut().unwrap().append_commands(vec![
                                        &format!("execute if score {} registers matches 0 run return run function {}:{} with storage memory:temp", reg, self.working_namespace.as_ref().unwrap().name(), label_following.name()),
                                        &format!("function {}:{} with storage memory:temp", self.working_namespace.as_ref().unwrap().name(), label_if_branch.name()),
                                    ]);
                                    // if branch
                                    self.work_with_next_mcfunction(label_if_branch);
                                    self.variable_table.enter_scope();
                                    self.generate_from_block(if_branch);
                                    self.variable_table.leave_scope();
                                    self.working_mcfunction.as_mut().unwrap().append_command(
                                        &format!(
                                            "function {}:{} with storage memory:temp",
                                            self.working_namespace.as_ref().unwrap().name(),
                                            label_following.name()
                                        ),
                                    );
                                    // following
                                    self.work_with_next_mcfunction(label_following);
                                }
                            }
                            _ => unreachable!(),
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
                                self.working_namespace.as_ref().unwrap().name(),
                                label_judge.name()
                            )]);
                        // judge
                        self.work_with_next_mcfunction(label_judge);
                        let exp_val = self.generate_from_exp(exp, &mut 0, &mut 0, &mut 0);
                        match exp_val {
                            ExpVal::Int { reg } => {
                                self.working_mcfunction.as_mut().unwrap().append_commands(vec![
                                    &format!("execute if score {} registers matches 0 run return run function {}:{} with storage memory:temp", reg, self.working_namespace.as_ref().unwrap().name(), label_following.name()),
                                    &format!("function {}:{} with storage memory:temp", self.working_namespace.as_ref().unwrap().name(), label_while_body.name()),
                                ]);
                                // while body
                                self.work_with_next_mcfunction(label_while_body);
                                self.variable_table.enter_scope();
                                self.generate_from_block(body);
                                self.variable_table.leave_scope();
                                self.working_mcfunction
                                    .as_mut()
                                    .unwrap()
                                    .append_commands(vec![
                                        "",
                                        &format!(
                                            "function {}:{} with storage memory:temp",
                                            self.working_namespace.as_ref().unwrap().name(),
                                            label_judge_name
                                        ),
                                    ]);
                                // following
                                self.break_labels.pop();
                                self.continue_labels.pop();
                                self.work_with_next_mcfunction(label_following);
                            }
                            _ => unreachable!(),
                        }
                    }
                    Stmt::Exp(exp) => {
                        self.generate_from_exp(exp, &mut 0, &mut 0, &mut 0);
                    }
                    Stmt::Break => {
                        self.working_mcfunction
                            .as_mut()
                            .unwrap()
                            .append_command(&format!(
                                "return run function {}:{} with storage memory:temp",
                                self.working_namespace.as_ref().unwrap().name(),
                                self.break_labels.last().unwrap(),
                            ));
                    }
                    Stmt::Continue => {
                        self.working_mcfunction
                            .as_mut()
                            .unwrap()
                            .append_command(&format!(
                                "return run function {}:{} with storage memory:temp",
                                self.working_namespace.as_ref().unwrap().name(),
                                self.continue_labels.last().unwrap(),
                            ));
                    }
                },
            }
        }
    }

    fn generate_from_exp(
        &mut self,
        exp: &mut Exp,
        reg_acc: &mut u32,
        path_acc: &mut u32,
        arr_acc: &mut u32,
    ) -> ExpVal {
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
                ExpVal::Int { reg: reg_res }
            }
            Exp::Variable(ident) => {
                let (is_local, variable) = self.variable_table.query_variable(ident);
                let decorated_name = variable.decorated_name;

                match &variable.data_type {
                    DataType::Int => {
                        let reg_res = format!("r{}", reg_acc);
                        if is_local {
                            self.working_mcfunction.as_mut().unwrap().append_command(&format!(
                                "$execute store result score {} registers run data get storage memory:stack frame[$(base_index)].{}",
                                reg_res, decorated_name
                            ));
                        } else {
                            panic!();
                        }
                        *reg_acc += 1;
                        ExpVal::Int { reg: reg_res }
                    }
                    DataType::Array { element_type } => {
                        if is_local {
                            let path_path = Path(
                                "memory:stack".into(),
                                format!("frame[$(base_index)].%path{}", path_acc),
                            );
                            *path_acc += 1;
                            self.working_mcfunction.as_mut().unwrap().append_command(&format!(
                                "$data modify storage {} {} set value \"memory:stack frame[$(base_index)].{}\"",
                                path_path.0,
                                path_path.1,
                                decorated_name
                            ));
                            ExpVal::Array {
                                element_type: *element_type.clone(),
                                path_path,
                            }
                        } else {
                            panic!();
                        }
                    }
                }
            }
            Exp::UnaryExp(op, exp) => {
                let exp_val = self.generate_from_exp(exp, reg_acc, path_acc, arr_acc);
                match exp_val {
                    ExpVal::Int { reg } => {
                        let reg_res = format!("r{}", reg_acc);
                        *reg_acc += 1;
                        match op {
                            UnaryOp::Positive => {
                                self.working_mcfunction
                                    .as_mut()
                                    .unwrap()
                                    .append_command(&format!(
                                        "scoreboard players operation {} registers = {} registers",
                                        reg_res, reg
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
                                        reg_res, reg
                                    ));
                            }
                        }
                        ExpVal::Int { reg: reg_res }
                    }
                    _ => unreachable!(),
                }
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
                let lhs_val = self.generate_from_exp(lhs, reg_acc, path_acc, arr_acc);
                let rhs_val = self.generate_from_exp(rhs, reg_acc, path_acc, arr_acc);

                match lhs_val {
                    ExpVal::Int { reg: reg_lhs } => {
                        match rhs_val {
                            ExpVal::Int { reg: reg_rhs } => {
                                let reg_res = format!("r{}", reg_acc);
                                if !is_rel {
                                    self.working_mcfunction.as_mut().unwrap().append_command(
                                        &format!(
                                        "scoreboard players operation {} registers = {} registers",
                                        reg_res, reg_lhs
                                    ),
                                    );
                                    self.working_mcfunction.as_mut().unwrap().append_command(
                                        &format!(
                                        "scoreboard players operation {} registers {} {} registers",
                                        reg_res, op, reg_rhs
                                    ),
                                    );
                                } else if !is_ne {
                                    self.working_mcfunction.as_mut().unwrap().append_command(
                                        &format!("scoreboard players set {} registers 0", reg_res),
                                    );
                                    self.working_mcfunction.as_mut().unwrap().append_command(&format!(
                                        "execute if score {} registers {} {} registers run scoreboard players set {} registers 1",
                                        reg_lhs, op, reg_rhs, reg_res
                                    ));
                                } else {
                                    self.working_mcfunction.as_mut().unwrap().append_command(
                                        &format!("scoreboard players set {} registers 1", reg_res),
                                    );
                                    self.working_mcfunction.as_mut().unwrap().append_command(&format!(
                                        "execute if score {} registers {} {} registers run scoreboard players set {} registers 0",
                                        reg_lhs, op, reg_rhs, reg_res
                                    ));
                                }
                                *reg_acc += 1;
                                ExpVal::Int { reg: reg_res }
                            }
                            _ => unreachable!(),
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Exp::FuncCall {
                namespace,
                func_ident,
                arguments,
            } => {
                let namespace = if namespace.is_some() {
                    namespace.clone().unwrap()
                } else {
                    self.working_namespace.as_ref().unwrap().name().to_owned()
                };
                let func_def = self
                    .function_table
                    .query_function((&namespace, func_ident))
                    .clone();
                // save registers
                for i in 0..*reg_acc {
                    self.working_mcfunction.as_mut().unwrap().append_command(
                        &format!("$execute store result storage memory:stack frame[$(base_index)].%r{} int 1.0 run scoreboard players get r{} registers", i, i)
                    );
                }
                // calculate arguments
                let mut reg_acc_1 = 0;
                for (i, arg) in arguments.iter_mut().enumerate() {
                    let exp_val = self.generate_from_exp(arg, &mut reg_acc_1, path_acc, arr_acc);
                    match exp_val {
                        ExpVal::Int { reg } => match func_def.params[i].data_type {
                            DataType::Int => {
                                self.working_mcfunction.as_mut().unwrap().append_command(
                                    &format!("execute store result storage memory:temp arguments.%{} int 1.0 run scoreboard players get {} registers",
                                    i, reg
                                ));
                            }
                            _ => unreachable!(),
                        },
                        ExpVal::Array {
                            element_type,
                            path_path,
                        } => match &func_def.params[i].data_type {
                            DataType::Array {
                                element_type: param_element_type,
                            } => {
                                if element_type != *param_element_type.clone() {
                                    panic!();
                                }
                                self.working_mcfunction.as_mut().unwrap().append_commands(vec![
                                    &format!("data modify storage memory:temp target_path set value \"memory:temp arguments.%{}\"", i),
                                    &format!("$data modify storage memory:temp src_path set from storage {} {}", path_path.0, path_path.1),
                                    &format!("function mcscript:mov_m_m with storage memory:temp")
                                ]);
                            }
                            _ => unreachable!(),
                        },
                    }
                }
                // call function
                self.working_mcfunction
                    .as_mut()
                    .unwrap()
                    .append_command(&format!(
                        "function {}:{} with storage memory:temp",
                        namespace, func_ident
                    ));
                // restore registers
                for i in 0..*reg_acc {
                    self.working_mcfunction.as_mut().unwrap().append_command(
                        &format!("$execute store result score r{} registers run data get storage memory:stack frame[$(base_index)].%r{}", i, i)
                    );
                }
                // store return value
                if func_def.func_type.is_some() {
                    match func_def.func_type.as_ref().unwrap() {
                        DataType::Int => {
                            let reg_res = format!("r{}", reg_acc);
                            self.working_mcfunction.as_mut().unwrap().append_command(
                                &format!(
                                    "scoreboard players operation {} registers = return_value registers",
                                    reg_res
                                ),
                            );
                            *reg_acc += 1;
                            ExpVal::Int { reg: reg_res }
                        }
                        DataType::Array { element_type } => {
                            let arr_path = Path(
                                "memory:stack".into(),
                                format!("frame[$(base_index)].%arr{}", arr_acc),
                            );
                            let path_path = Path(
                                "memory:stack".into(),
                                format!("frame[$(base_index)].%path{}", path_acc),
                            );
                            *arr_acc += 1;
                            *path_acc += 1;
                            self.working_mcfunction
                                .as_mut()
                                .unwrap()
                                .append_commands(vec![
                                    &format!("$data modify storage {} {} set from storage memory:temp return_array", arr_path.0, arr_path.1),
                                    &format!("$data modify storage {} {} set value \"{} {}\"", path_path.0, path_path.1, arr_path.0, arr_path.1),
                                ]);
                            ExpVal::Array {
                                element_type: *element_type.clone(),
                                path_path,
                            }
                        }
                    }
                } else {
                    ExpVal::Int { reg: "".into() }
                }
            }
        }
    }

    fn new_label(&mut self) -> Mcfunction {
        let mcfunction_name = format!("{}-label_{}", self.working_function_ident, self.label_acc);
        self.label_acc += 1;
        Mcfunction::new(mcfunction_name)
    }

    fn work_with_next_mcfunction(&mut self, next_mcfunction: Mcfunction) {
        self.working_namespace
            .as_mut()
            .unwrap()
            .append_mcfunction(self.working_mcfunction.take().unwrap());
        self.working_mcfunction = Some(next_mcfunction);
    }
}
