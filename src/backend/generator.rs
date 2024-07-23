use std::{collections::HashMap, fmt::Display};

use crate::{
    backend::datapack::{Datapack, Mcfunction, Namespace},
    frontend::ast::{
        exp::{BinaryOp, Exp, UnaryOp},
        Block, BlockItem, CompileUnit, DataType, FuncDef, FuncParam, GlobalDef, Ident, Stmt,
    },
};

use super::error::SemanticError;

#[derive(Clone)]
struct Variable {
    is_local: bool,
    decorated_name: String,
    data_type: DataType,
}

impl Variable {
    pub fn memory_location(&self) -> Location {
        if self.is_local {
            Location::Memory(
                "memory:stack".into(),
                format!("frame[$(base_index)].{}", self.decorated_name),
            )
        } else {
            Location::Memory("memory:global".into(), format!("{}", self.decorated_name))
        }
    }
}

struct VariableTable(
    Vec<HashMap<String, Variable>>,
    HashMap<(String, String), Variable>,
);

impl VariableTable {
    pub fn new() -> Self {
        Self(vec![HashMap::new()], HashMap::new())
    }

    pub fn enter_scope(&mut self) {
        self.0.push(HashMap::<_, _>::new());
    }

    pub fn leave_scope(&mut self) {
        self.0.pop();
    }

    pub fn new_local_variable(
        &mut self,
        ident: &Ident,
        data_type: DataType,
    ) -> Result<Variable, SemanticError> {
        if self.0.last().unwrap().contains_key(&ident.string) {
            return Err(SemanticError::MultipleDefinition {
                ident: ident.string.clone(),
                begin: ident.src_loc.begin,
                end: ident.src_loc.end,
            });
        }
        let decorated_name = format!("{}@{}", ident.string, self.0.len() - 1);
        let variable = Variable {
            is_local: true,
            decorated_name: decorated_name.clone(),
            data_type,
        };
        self.0
            .last_mut()
            .unwrap()
            .insert(ident.string.to_owned(), variable.clone());
        Ok(variable)
    }

    pub fn new_global_variable(
        &mut self,
        ident: &Ident,
        namespace: &str,
        data_type: DataType,
    ) -> Result<Variable, SemanticError> {
        let key = (ident.string.to_owned(), namespace.to_owned());
        if self.1.contains_key(&key) {
            return Err(SemanticError::MultipleDefinition {
                ident: ident.string.to_owned(),
                begin: ident.src_loc.begin,
                end: ident.src_loc.end,
            });
        }
        let decorated_name = format!("{}@{}", ident.string, namespace);
        let variable = Variable {
            is_local: false,
            decorated_name,
            data_type,
        };
        self.1.insert(key, variable.clone());
        Ok(variable)
    }

    pub fn set_parameters(&mut self, params: &Vec<FuncParam>) {
        for (i, param) in params.iter().enumerate() {
            let decorated_name = format!("%{}", i);
            self.0.last_mut().unwrap().insert(
                param.ident.string.clone(),
                Variable {
                    is_local: true,
                    decorated_name,
                    data_type: param.data_type.clone(),
                },
            );
        }
    }

    pub fn query_variable(
        &self,
        ident: &Ident,
        namespace: &Option<String>,
        current_namespace: &str,
    ) -> Result<Variable, SemanticError> {
        if namespace.is_none() {
            for scope in self.0.iter().rev() {
                if scope.contains_key(&ident.string) {
                    return Ok(scope[&ident.string].clone());
                }
            }
            let key = (ident.string.to_owned(), current_namespace.to_owned());
            if self.1.contains_key(&key) {
                return Ok(self.1[&key].clone());
            }
            Err(SemanticError::UndefinedIdentifier {
                ident: ident.string.to_owned(),
                begin: ident.src_loc.begin,
                end: ident.src_loc.end,
            })
        } else {
            let key = (
                ident.string.to_owned(),
                namespace.as_ref().unwrap().to_owned(),
            );
            if self.1.contains_key(&key) {
                return Ok(self.1[&key].clone());
            }
            Err(SemanticError::UndefinedIdentifier {
                ident: ident.string.to_owned(),
                begin: ident.src_loc.begin,
                end: ident.src_loc.end,
            })
        }
    }
}

struct FunctionTable(HashMap<(String, String), FuncDef>);

impl FunctionTable {
    pub fn new() -> Self {
        Self(HashMap::<_, _>::new())
    }

    pub fn new_function(
        &mut self,
        namespace: &str,
        ident: &Ident,
        func_def: FuncDef,
    ) -> Result<(), SemanticError> {
        let key = (namespace.to_owned(), ident.string.to_owned());
        if self.0.contains_key(&key) {
            return Err(SemanticError::MultipleDefinition {
                ident: ident.string.to_owned(),
                begin: ident.src_loc.begin,
                end: ident.src_loc.end,
            });
        }
        self.0.insert(key, func_def);
        Ok(())
    }

    pub fn query_function(
        &self,
        namespace: &str,
        ident: &Ident,
    ) -> Result<&FuncDef, SemanticError> {
        match self.0.get(&(namespace.to_owned(), ident.string.to_owned())) {
            Some(func_def) => Ok(func_def),
            None => Err(SemanticError::UndefinedIdentifier {
                ident: ident.string.to_owned(),
                begin: ident.src_loc.begin,
                end: ident.src_loc.end,
            }),
        }
    }
}

struct ExpVal {
    data_type: DataType,
    location: Location,
}

#[derive(Clone)]
enum Location {
    Register(String),
    Memory(String, String),
    MemoryRef(String, String),
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Location::Register(reg) => write!(f, "{}", reg),
            Location::Memory(naid, path) => write!(f, "{} {}", naid, path),
            Location::MemoryRef(naid, path) => write!(f, "{} {}", naid, path),
        }
    }
}

impl Location {
    fn return_value() -> Self {
        Self::Memory("memory:temp".into(), "return_value".into())
    }

    fn argument(i: u32) -> Self {
        Self::Memory("memory:temp".into(), format!("arguments.%{}", i))
    }

    fn memory_ref(ref_location: Location) -> Self {
        if let Location::Memory(naid, path) = ref_location {
            Self::MemoryRef(naid, path)
        } else {
            panic!()
        }
    }
}

struct RegAcc {
    cnt: u32,
}

impl RegAcc {
    pub fn new() -> Self {
        Self { cnt: 0 }
    }

    pub fn new_reg(&mut self) -> Location {
        let reg = Location::Register(format!("r{}", self.cnt));
        self.cnt += 1;
        reg
    }
}

struct ObjAcc {
    cnt: u32,
}

impl ObjAcc {
    pub fn new() -> Self {
        Self { cnt: 0 }
    }

    pub fn new_obj(&mut self) -> Location {
        let obj = Location::Memory(
            "memory:stack".into(),
            format!("frame[$(base_index)].%obj{}", self.cnt),
        );
        self.cnt += 1;
        obj
    }
}

pub struct Generator {
    variable_table: VariableTable,
    function_table: FunctionTable,
    datapack: Datapack,
    working_namespace: Option<Namespace>,
    working_function_ident: String,
    working_mcfunction: Option<Mcfunction>,
    label_acc: u32,
    custom_cmd_acc: u32,
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
            custom_cmd_acc: 0,
            break_labels: vec![],
            continue_labels: vec![],
        }
    }

    pub fn generate(
        &mut self,
        compile_units: Vec<(CompileUnit, String)>,
    ) -> Result<Datapack, (String, SemanticError)> {
        for (compile_unit, namespace) in &compile_units {
            self.scan_global_defs(compile_unit, namespace)
                .map_err(|err| (namespace.to_owned(), err))?;
        }
        for (compile_unit, namespace) in compile_units {
            self.generate_from_namespace(compile_unit, namespace.clone())
                .map_err(|err| (namespace.clone(), err))?;
        }
        Ok(self.datapack.clone())
    }

    fn scan_global_defs(
        &mut self,
        compile_unit: &CompileUnit,
        namespace: &str,
    ) -> Result<(), SemanticError> {
        for global_def in &compile_unit.global_defs {
            match global_def {
                GlobalDef::FuncDef(func_def) => {
                    self.function_table.new_function(
                        namespace,
                        &func_def.ident,
                        func_def.clone(),
                    )?;
                }
                GlobalDef::VariableDef {
                    ident,
                    init_value: _,
                    data_type,
                } => {
                    self.variable_table
                        .new_global_variable(ident, namespace, data_type.clone())?;
                }
            }
        }
        Ok(())
    }

    fn generate_from_namespace(
        &mut self,
        mut compile_unit: CompileUnit,
        namespace: String,
    ) -> Result<(), SemanticError> {
        self.working_namespace = Some(Namespace::new(namespace.clone()));

        // handle global variable definitions
        let mut init = Mcfunction::new("init".into());
        init.append_prologue();
        init.append_command(&format!(
            "function {}:init-label_0 with storage memory:temp",
            namespace,
        ));
        init.append_epilogue();
        self.working_namespace().append_mcfunction(init);
        self.working_function_ident = "init".into();
        self.label_acc = 0;
        self.working_mcfunction = Some(self.new_label());
        for global_def in &mut compile_unit.global_defs {
            match global_def {
                GlobalDef::VariableDef {
                    ident,
                    init_value,
                    data_type,
                } => {
                    let variable = self.variable_table.query_variable(
                        ident,
                        &Some(namespace.clone()),
                        &namespace,
                    )?;
                    let exp_val = self.eval(init_value, &mut RegAcc::new(), &mut ObjAcc::new())?;
                    if &exp_val.data_type != data_type {
                        panic!();
                    }
                    self.mov(&variable.memory_location(), &exp_val.location);
                }
                _ => {}
            }
        }
        self.working_namespace
            .as_mut()
            .unwrap()
            .append_mcfunction(self.working_mcfunction.take().unwrap());

        // generate functions
        self.custom_cmd_acc = 0;
        for global_def in &mut compile_unit.global_defs {
            if let GlobalDef::FuncDef(func_def) = global_def {
                self.generate_from_func_def(func_def)?;
            }
        }

        self.datapack
            .append_namespace(self.working_namespace.take().unwrap());
        Ok(())
    }

    fn generate_from_func_def(&mut self, func_def: &mut FuncDef) -> Result<(), SemanticError> {
        self.working_function_ident = func_def.ident.string.clone();

        let mut entry = Mcfunction::new(self.working_function_ident.clone());
        entry.append_prologue();
        entry.append_command(&format!(
            "function {}:{}-label_0 with storage memory:temp",
            self.working_namespace.as_ref().unwrap().name(),
            self.working_function_ident.clone()
        ));
        entry.append_epilogue();
        self.working_namespace
            .as_mut()
            .unwrap()
            .append_mcfunction(entry);

        self.label_acc = 0;
        self.working_mcfunction = Some(self.new_label());
        self.variable_table.enter_scope();
        self.variable_table.set_parameters(&func_def.params);
        self.generate_from_block(&mut func_def.block, &func_def.func_type)?;
        self.variable_table.leave_scope();

        self.working_namespace
            .as_mut()
            .unwrap()
            .append_mcfunction(self.working_mcfunction.take().unwrap());
        Ok(())
    }

    fn generate_from_block(
        &mut self,
        block: &mut Block,
        expected_return_type: &Option<DataType>,
    ) -> Result<(), SemanticError> {
        for block_item in &mut block.0 {
            match block_item {
                BlockItem::Decl(decl) => {
                    let exp_val =
                        self.eval(&mut decl.init_value, &mut RegAcc::new(), &mut ObjAcc::new())?;
                    let variable = self
                        .variable_table
                        .new_local_variable(&decl.ident, exp_val.data_type)?;
                    self.mov(&variable.memory_location(), &exp_val.location);
                }
                BlockItem::Stmt(stmt) => {
                    match stmt {
                        Stmt::Return { return_value } => {
                            if return_value.is_some() {
                                let return_value = return_value.as_mut().unwrap();
                                let exp_val = self.eval(
                                    return_value,
                                    &mut RegAcc::new(),
                                    &mut ObjAcc::new(),
                                )?;
                                if &exp_val.data_type != expected_return_type.as_ref().unwrap() {
                                    panic!();
                                }
                                self.mov(&Location::return_value(), &exp_val.location);
                                self.working_mcfunction().append_command("return 0");
                            } else {
                                if expected_return_type.is_some() {
                                    panic!();
                                } else {
                                    self.working_mcfunction().append_command("return 0");
                                }
                            }
                        }
                        Stmt::Assign { lhs, new_value } => {
                            let mut reg_acc = RegAcc::new();
                            let mut obj_acc = ObjAcc::new();
                            let rhs_val = self.eval(new_value, &mut reg_acc, &mut obj_acc)?;
                            let lhs_val = self.eval(lhs, &mut reg_acc, &mut obj_acc)?;
                            if lhs_val.data_type != rhs_val.data_type {
                                panic!();
                            }
                            self.mov(&lhs_val.location, &rhs_val.location);
                        }
                        Stmt::Block(block) => {
                            self.variable_table.enter_scope();
                            self.generate_from_block(block, expected_return_type)?;
                            self.variable_table.leave_scope();
                        }
                        Stmt::IfElse {
                            exp,
                            if_branch,
                            else_branch,
                        } => {
                            let mut reg_acc = RegAcc::new();
                            let exp_val = self.eval(exp, &mut reg_acc, &mut ObjAcc::new())?;
                            if exp_val.data_type != DataType::Int {
                                panic!();
                            }
                            let reg = self.to_reg_readonly(&exp_val.location, &mut reg_acc);

                            let namespace = self.working_namespace_name().to_owned();
                            let label_if_branch = self.new_label();
                            match else_branch {
                                Some(else_branch) => {
                                    let label_else_branch = self.new_label();
                                    let label_following = self.new_label();
                                    self.working_mcfunction().append_commands(vec![
                                        &format!(
                                            "execute if score {} registers matches 0 run return run function {}:{} with storage memory:temp", 
                                            reg, namespace, label_else_branch.name()
                                        ),
                                        &format!("function {}:{} with storage memory:temp", namespace, label_if_branch.name()),
                                    ]);
                                    // if branch
                                    self.work_with_next_mcfunction(label_if_branch);
                                    self.variable_table.enter_scope();
                                    self.generate_from_block(if_branch, expected_return_type)?;
                                    self.variable_table.leave_scope();
                                    self.working_mcfunction().append_command(&format!(
                                        "function {}:{} with storage memory:temp",
                                        namespace,
                                        label_following.name()
                                    ));
                                    // else branch
                                    self.work_with_next_mcfunction(label_else_branch);
                                    self.variable_table.enter_scope();
                                    self.generate_from_block(
                                        &mut else_branch.clone(),
                                        expected_return_type,
                                    )?;
                                    self.variable_table.leave_scope();
                                    self.working_mcfunction().append_command(&format!(
                                        "function {}:{} with storage memory:temp",
                                        namespace,
                                        label_following.name()
                                    ));
                                    // following
                                    self.work_with_next_mcfunction(label_following);
                                }
                                None => {
                                    let label_following = self.new_label();
                                    self.working_mcfunction().append_commands(vec![
                                        &format!(
                                            "execute if score {} registers matches 0 run return run function {}:{} with storage memory:temp", 
                                            reg, namespace, label_following.name()
                                        ),
                                        &format!("function {}:{} with storage memory:temp", namespace, label_if_branch.name()),
                                    ]);
                                    // if branch
                                    self.work_with_next_mcfunction(label_if_branch);
                                    self.variable_table.enter_scope();
                                    self.generate_from_block(if_branch, expected_return_type)?;
                                    self.variable_table.leave_scope();
                                    self.working_mcfunction().append_command(&format!(
                                        "function {}:{} with storage memory:temp",
                                        namespace,
                                        label_following.name()
                                    ));
                                    // following
                                    self.work_with_next_mcfunction(label_following);
                                }
                            }
                        }
                        Stmt::While { exp, body } => {
                            let label_judge = self.new_label();
                            let label_while_body = self.new_label();
                            let label_following = self.new_label();

                            let label_judge_name = label_judge.name().to_owned();
                            let namespace = self.working_namespace_name().to_owned();

                            self.break_labels.push(label_following.name().to_owned());
                            self.continue_labels.push(label_judge.name().to_owned());

                            self.working_mcfunction().append_commands(vec![&format!(
                                "function {}:{} with storage memory:temp",
                                namespace,
                                label_judge.name()
                            )]);

                            // judge
                            self.work_with_next_mcfunction(label_judge);
                            let mut reg_acc = RegAcc::new();
                            let exp_val = self.eval(exp, &mut reg_acc, &mut ObjAcc::new())?;
                            if exp_val.data_type != DataType::Int {
                                panic!();
                            }
                            let reg = self.to_reg_readonly(&exp_val.location, &mut reg_acc);
                            self.working_mcfunction().append_commands(vec![
                                &format!(
                                    "execute if score {} registers matches 0 run return run function {}:{} with storage memory:temp", 
                                    reg, namespace, label_following.name()
                                ),
                                &format!("function {}:{} with storage memory:temp", namespace, label_while_body.name()),
                            ]);
                            // while body
                            self.work_with_next_mcfunction(label_while_body);
                            self.variable_table.enter_scope();
                            self.generate_from_block(body, expected_return_type)?;
                            self.variable_table.leave_scope();
                            self.working_mcfunction().append_commands(vec![
                                "",
                                &format!(
                                    "function {}:{} with storage memory:temp",
                                    namespace, label_judge_name
                                ),
                            ]);
                            // following
                            self.break_labels.pop();
                            self.continue_labels.pop();
                            self.work_with_next_mcfunction(label_following);
                        }
                        Stmt::Exp(exp) => {
                            self.eval(exp, &mut RegAcc::new(), &mut ObjAcc::new())?;
                        }
                        Stmt::Break => {
                            let break_label = self.break_labels.last().unwrap().clone();
                            let namespace = self.working_namespace_name().to_owned();
                            self.working_mcfunction().append_command(&format!(
                                "return run function {}:{} with storage memory:temp",
                                namespace, break_label
                            ));
                        }
                        Stmt::Continue => {
                            let continue_label = self.continue_labels.last().unwrap().clone();
                            let namespace = self.working_namespace_name().to_owned();
                            self.working_mcfunction().append_command(&format!(
                                "return run function {}:{} with storage memory:temp",
                                namespace, continue_label
                            ));
                        }
                        Stmt::InlineCommand { fmt_str, arguments } => {
                            for (i, arg) in arguments.iter_mut().enumerate() {
                                let exp_val =
                                    self.eval(arg, &mut RegAcc::new(), &mut ObjAcc::new())?;
                                self.mov(
                                    &Location::Memory(
                                        "memory:temp".into(),
                                        format!("custom_command_arguments.{}", i),
                                    ),
                                    &exp_val.location,
                                );
                            }
                            let mut custom_cmd =
                                Mcfunction::new(format!("custom_cmd_{}", self.custom_cmd_acc));
                            self.custom_cmd_acc += 1;
                            let mut cmd = fmt_str.to_owned();
                            let mut i = 0;
                            while cmd.contains("{}") {
                                cmd = cmd.replacen("{}", &format!("$({})", i), 1);
                                i += 1;
                            }
                            custom_cmd.append_command(&cmd);
                            let custom_cmd_name = custom_cmd.name().to_owned();
                            let namespace = self.working_namespace_name().to_owned();
                            self.working_namespace().append_mcfunction(custom_cmd);
                            self.working_mcfunction().append_commands(vec![&format!(
                                "function {}:{} with storage memory:temp custom_command_arguments",
                                namespace, custom_cmd_name
                            )]);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn get_element(
        &mut self,
        array: &mut Exp,
        subscript: &mut Exp,
        reg_acc: &mut RegAcc,
        obj_acc: &mut ObjAcc,
    ) -> Result<ExpVal, SemanticError> {
        let subscript_val = self.eval(subscript, reg_acc, obj_acc)?;
        if subscript_val.data_type != DataType::Int {
            panic!();
        }

        let arr_val = self.eval(array, reg_acc, obj_acc)?;
        if let DataType::Array { element_type } = arr_val.data_type {
            self.mov(
                &Location::Memory("memory:temp".into(), "subscript".into()),
                &subscript_val.location,
            );
            match arr_val.location {
                Location::Memory(naid, path) => {
                    self.working_mcfunction().append_command(&format!(
                        "data modify storage memory:temp array_path set value \"{} {}\"",
                        naid, path
                    ));
                }
                Location::MemoryRef(loc_naid, loc_path) => {
                    self.working_mcfunction().append_command(&format!(
                        "data modify storage memory:temp array_path set from storage {} {}",
                        loc_naid, loc_path
                    ));
                }
                _ => unreachable!(),
            }
            self.working_mcfunction()
                .append_command("function mcscript:load_element_path with storage memory:temp");
            let element_location = obj_acc.new_obj();
            self.mov(
                &element_location,
                &Location::Memory("memory:temp".into(), "element_path".into()),
            );
            Ok(ExpVal {
                data_type: *element_type,
                location: Location::memory_ref(element_location),
            })
        } else {
            panic!()
        }
    }

    fn eval(
        &mut self,
        exp: &mut Exp,
        reg_acc: &mut RegAcc,
        obj_acc: &mut ObjAcc,
    ) -> Result<ExpVal, SemanticError> {
        match exp {
            Exp::Number(num) => {
                let reg_res = reg_acc.new_reg();
                self.mov_immediate(&reg_res, &num.to_string(), obj_acc);
                Ok(ExpVal {
                    data_type: DataType::Int,
                    location: reg_res,
                })
            }
            Exp::Variable { ident, namespace } => {
                let variable = self.variable_table.query_variable(
                    ident,
                    &namespace.as_ref().map(|n| n.string.to_owned()),
                    self.working_namespace_name(),
                )?;
                Ok(ExpVal {
                    data_type: variable.data_type.clone(),
                    location: variable.memory_location(),
                })
            }
            Exp::UnaryExp(op, exp) => {
                let exp_val = self.eval(exp, reg_acc, obj_acc)?;
                if let DataType::Int = exp_val.data_type {
                    let reg_exp = reg_acc.new_reg();
                    self.mov(&reg_exp, &exp_val.location);
                    match op {
                        UnaryOp::Positive => Ok(ExpVal {
                            data_type: DataType::Int,
                            location: reg_exp,
                        }),
                        UnaryOp::Negative => {
                            let reg_res = reg_acc.new_reg();
                            self.mov_immediate(&reg_res, "0", obj_acc);
                            self.working_mcfunction().append_command(&format!(
                                "scoreboard players operation {} registers -= {} registers",
                                reg_res, reg_exp
                            ));
                            Ok(ExpVal {
                                data_type: DataType::Int,
                                location: reg_res,
                            })
                        }
                    }
                } else {
                    panic!();
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
                let lhs_val = self.eval(lhs, reg_acc, obj_acc)?;
                let rhs_val = self.eval(rhs, reg_acc, obj_acc)?;

                if let DataType::Int = lhs_val.data_type {
                    if let DataType::Int = rhs_val.data_type {
                        let reg_rhs = self.to_reg_readonly(&rhs_val.location, reg_acc);
                        let reg_res = reg_acc.new_reg();
                        if !is_rel {
                            self.mov(&reg_res, &lhs_val.location);
                            self.working_mcfunction().append_command(&format!(
                                "scoreboard players operation {} registers {} {} registers",
                                reg_res, op, reg_rhs
                            ));
                        } else {
                            let reg_lhs = self.to_reg_readonly(&lhs_val.location, reg_acc);
                            if !is_ne {
                                self.mov_immediate(&reg_res, "0", obj_acc);
                                self.working_mcfunction().append_command(
                                    &format!(
                                        "execute if score {} registers {} {} registers run scoreboard players set {} registers 1",
                                        reg_lhs, op, reg_rhs, reg_res
                                    )
                                );
                            } else {
                                self.mov_immediate(&reg_res, "1", obj_acc);
                                self.working_mcfunction().append_command(
                                    &format!(
                                        "execute if score {} registers {} {} registers run scoreboard players set {} registers 0",
                                        reg_lhs, op, reg_rhs, reg_res
                                    )
                                );
                            }
                        }
                        return Ok(ExpVal {
                            data_type: DataType::Int,
                            location: reg_res,
                        });
                    }
                }
                panic!();
            }
            Exp::FuncCall {
                namespace,
                func_ident,
                arguments,
            } => {
                let namespace = match namespace {
                    Some(namespace) => namespace.string.to_owned(),
                    None => self.working_namespace_name().to_owned(),
                };

                let func_def = self
                    .function_table
                    .query_function(&namespace, func_ident)?
                    .clone();

                // save registers
                for i in 0..reg_acc.cnt {
                    self.mov(
                        &Location::Memory(
                            "memory:stack".into(),
                            format!("frame[$(base_index)].%r{}", i),
                        ),
                        &Location::Register(format!("r{}", i)),
                    );
                }
                // calculate arguments
                let mut reg_acc_1 = RegAcc::new(); // We just saved the using registers, so here we can restart the register counting.
                for (i, arg) in arguments.iter_mut().enumerate() {
                    let exp_val = self.eval(arg, &mut reg_acc_1, obj_acc)?;
                    self.mov(&Location::argument(i as u32), &exp_val.location);
                }
                // call function
                self.working_mcfunction().append_command(&format!(
                    "function {}:{} with storage memory:temp",
                    namespace, func_ident.string
                ));
                // restore registers
                for i in 0..reg_acc.cnt {
                    self.mov(
                        &Location::Register(format!("r{}", i)),
                        &Location::Memory(
                            "memory:stack".into(),
                            format!("frame[$(base_index)].%r{}", i),
                        ),
                    );
                }
                // store return value
                if func_def.func_type.is_some() {
                    let data_type = func_def.func_type.as_ref().unwrap();
                    match data_type {
                        DataType::Int => {
                            let reg_res = reg_acc.new_reg();
                            self.mov(&reg_res, &Location::return_value());
                            Ok(ExpVal {
                                data_type: data_type.clone(),
                                location: reg_res,
                            })
                        }
                        DataType::Array { element_type: _ } => {
                            let obj_res = obj_acc.new_obj();
                            self.mov(&obj_res, &Location::return_value());
                            Ok(ExpVal {
                                data_type: data_type.clone(),
                                location: obj_res,
                            })
                        }
                    }
                } else {
                    Ok(ExpVal {
                        data_type: DataType::Int,
                        location: Location::return_value(),
                    })
                }
            }
            Exp::NewArray { length, element } => {
                let label_judge = self.new_label();
                let label_while_body = self.new_label();
                let label_following = self.new_label();

                let label_judge_name = label_judge.name().to_owned();

                let namespace = self.working_namespace_name().to_owned();

                let length_val = self.eval(length, reg_acc, obj_acc)?;
                let element_val = self.eval(element, reg_acc, obj_acc)?;

                if length_val.data_type != DataType::Int {
                    panic!();
                }

                let reg_len = self.to_reg_readonly(&length_val.location, reg_acc);
                let reg_current_len = reg_acc.new_reg();
                let arr = obj_acc.new_obj();
                self.mov_immediate(&arr, "[]", obj_acc);
                self.mov_immediate(&reg_current_len, "0", obj_acc);
                self.working_mcfunction().append_command(&format!(
                    "function {}:{} with storage memory:temp",
                    namespace,
                    label_judge.name(),
                ));
                // judge
                self.work_with_next_mcfunction(label_judge);
                self.working_mcfunction.as_mut().unwrap().append_commands(vec![
                    &format!(
                        "execute if score {} registers >= {} registers run return run function {}:{} with storage memory:temp", 
                        reg_current_len, reg_len, namespace, label_following.name()
                    ),
                    &format!("function {}:{} with storage memory:temp", namespace, label_while_body.name()),
                ]);
                // append
                self.work_with_next_mcfunction(label_while_body);
                self.mov(
                    &Location::Memory("memory:temp".into(), "element".into()),
                    &element_val.location,
                );
                self.working_mcfunction().append_command(&format!(
                    "data modify storage {} append from storage memory:temp element",
                    arr
                ));
                self.working_mcfunction().append_commands(vec![
                    &format!("scoreboard players add {} registers 1", reg_current_len),
                    "",
                    &format!(
                        "function {}:{} with storage memory:temp",
                        namespace, label_judge_name
                    ),
                ]);
                // following
                self.work_with_next_mcfunction(label_following);
                Ok(ExpVal {
                    data_type: DataType::Array {
                        element_type: Box::new(element_val.data_type),
                    },
                    location: arr,
                })
            }
            Exp::ArrayElement { array, subscript } => {
                self.get_element(array, subscript, reg_acc, obj_acc)
            }
            Exp::SquareBracketsArray {
                element_type,
                elements,
            } => {
                let arr = obj_acc.new_obj();
                self.mov_immediate(&arr, "[]", obj_acc);

                if elements.is_empty() {
                    return Ok(ExpVal {
                        data_type: DataType::Array {
                            element_type: Box::new(element_type.as_ref().unwrap().clone()),
                        },
                        location: arr,
                    });
                }

                let mut iter = elements.iter_mut();
                let first = iter.next().unwrap();
                let first_val = self.eval(first, reg_acc, obj_acc)?;
                let first_element_type = &first_val.data_type;
                if element_type.is_some() && element_type.as_ref().unwrap() != first_element_type {
                    panic!();
                }

                self.mov(
                    &Location::Memory("memory:temp".into(), "element".into()),
                    &first_val.location,
                );
                self.working_mcfunction().append_command(&format!(
                    "data modify storage {} append from storage memory:temp element",
                    arr
                ));

                for element in iter {
                    let element_val = self.eval(element, reg_acc, obj_acc)?;
                    if &element_val.data_type != first_element_type {
                        panic!();
                    }
                    self.mov(
                        &Location::Memory("memory:temp".into(), "element".into()),
                        &element_val.location,
                    );
                    self.working_mcfunction().append_command(&format!(
                        "data modify storage {} append from storage memory:temp element",
                        arr
                    ));
                }

                Ok(ExpVal {
                    data_type: DataType::Array {
                        element_type: Box::new(first_element_type.clone()),
                    },
                    location: arr,
                })
            }
        }
    }

    fn mov(&mut self, dest: &Location, src: &Location) {
        match src {
            Location::Register(reg_src) => match dest {
                Location::Register(reg_dest) => self.working_mcfunction().append_command(&format!(
                    "scoreboard players operation {} registers = {} registers",
                    reg_dest, reg_src
                )),
                Location::Memory(naid, path) => self.working_mcfunction().append_command(
                    &format!("execute store result storage {} {} int 1.0 run scoreboard players get {} registers", naid, path, reg_src
                )),
                Location::MemoryRef(naid, path) => {
                    self.working_mcfunction().append_commands(vec![
                        &format!("data modify storage memory:temp target_path set from storage {} {}", naid, path),
                        &format!("data modify storage memory:temp src_reg set value \"{}\"", reg_src),
                        "function mcscript:mov_m_r with storage memory:temp",
                    ])
                }
            },
            Location::Memory(naid_src, path_src) => match dest {
                Location::Register(reg_dest) => self.working_mcfunction().append_command(
                    &format!("execute store result score {} registers run data get storage {} {} 1.0", reg_dest, naid_src, path_src
                )),
                Location::Memory(naid_dest, path_dest) => self.working_mcfunction().append_command(&format!("data modify storage {} {} set from storage {} {}", naid_dest, path_dest, naid_src, path_src)),
                Location::MemoryRef(loc_naid_dest, loc_path_dest) => self.working_mcfunction().append_commands(vec![
                    &format!("data modify storage memory:temp target_path set from storage {} {}", loc_naid_dest, loc_path_dest),
                    &format!("data modify storage memory:temp src_path set value \"{} {}\"", naid_src, path_src),
                    "function mcscript:mov_m_m with storage memory:temp",
                ])
            },
            Location::MemoryRef(loc_naid_src, loc_path_src) => match dest {
                Location::Register(reg_dest) => self.working_mcfunction().append_commands(vec![
                    &format!("data modify storage memory:temp target_reg set value \"{}\"", reg_dest),
                    &format!("data modify storage memory:temp src_path set from storage {} {}", loc_naid_src, loc_path_src),
                    "function mcscript:mov_r_m with storage memory:temp",
                ]),
                Location::Memory(naid_dest, path_dest) => self.working_mcfunction().append_commands(vec![
                    &format!("data modify storage memory:temp target_path set value \"{} {}\"", naid_dest, path_dest),
                    &format!("data modify storage memory:temp src_path set from storage {} {}", loc_naid_src, loc_path_src),
                    "function mcscript:mov_m_m with storage memory:temp",
                ]),
                Location::MemoryRef(loc_naid_dest, loc_path_dest ) => self.working_mcfunction().append_commands(vec![
                    &format!("data modify storage memory:temp target_path set from storage {} {}", loc_naid_dest, loc_path_dest),
                    &format!("data modify storage memory:temp src_path set from storage {} {}", loc_naid_src, loc_path_src),
                    "function mcscript:mov_m_m with storage memory:temp",
                ]),
            }
        }
    }

    fn mov_immediate(&mut self, dest: &Location, src: &str, obj_acc: &mut ObjAcc) {
        match dest {
            Location::Register(reg_dest) => self.working_mcfunction().append_command(&format!(
                "scoreboard players set {} registers {}",
                reg_dest, src
            )),
            Location::Memory(naid_dest, path_dest) => {
                self.working_mcfunction().append_command(&format!(
                    "data modify storage {} {} set value {}",
                    naid_dest, path_dest, src
                ))
            }
            Location::MemoryRef(_, _) => {
                let obj = obj_acc.new_obj();
                self.mov_immediate(&obj, src, obj_acc);
                self.mov(dest, &obj);
            }
        }
    }

    fn to_reg_readonly(&mut self, src: &Location, reg_acc: &mut RegAcc) -> Location {
        match src {
            Location::Register(_) => src.clone(),
            _ => {
                let dest = reg_acc.new_reg();
                self.mov(&dest, src);
                dest
            }
        }
    }

    fn working_mcfunction(&mut self) -> &mut Mcfunction {
        self.working_mcfunction.as_mut().unwrap()
    }

    fn working_namespace(&mut self) -> &mut Namespace {
        self.working_namespace.as_mut().unwrap()
    }

    fn working_namespace_name(&self) -> &str {
        self.working_namespace.as_ref().unwrap().name()
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
