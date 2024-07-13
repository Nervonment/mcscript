use crate::{
    ast::{
        exp::{BinaryOp, Exp, UnaryOp},
        BlockItem, FuncDef, Program, Stmt,
    },
    datapack::{Datapack, Mcfunction, Namespace},
};

impl FuncDef {
    pub fn to_mcfunction(&self, dest: &mut Namespace) {
        let mut entry = Mcfunction::new(self.ident.clone());
        entry.append_commands(vec![
            "scoreboard players add base_index registers 1",
            "execute store result storage memory:temp base_index int 1.0 run scoreboard players get base_index registers",
            &format!("function {}:{}-push_frame with storage memory:temp", dest.name(), self.ident),
            "",
            &format!("function {}:{}-call_body with storage memory:temp", dest.name(), self.ident),
            "",
            &format!("function {}:util-pop_frame with storage memory:temp", dest.name()),
            ""
        ]);
        dest.append_mcfunction(entry);

        let mut push_frame = Mcfunction::new(format!("{}-push_frame", self.ident.clone()));
        push_frame.append_commands(vec![
            "data modify storage memory:stack frame append value {}",
            // local variables
            "$data modify storage memory:stack frame[$(base_index)] merge value {}",
            // arguments
            "$data modify storage memory:stack frame[$(base_index)] merge value {}",
            // base index
            "$data modify storage memory:stack frame[$(base_index)] merge value {base_index: $(base_index)}",
            ""
        ]);
        dest.append_mcfunction(push_frame);

        let mut call_body = Mcfunction::new(format!("{}-call_body", self.ident.clone()));
        call_body.append_command(&format!(
            "$function {}:{}-body with storage memory:stack frame[$(base_index)]",
            dest.name(),
            self.ident
        ));
        dest.append_mcfunction(call_body);

        let mut body = Mcfunction::new(format!("{}-body", self.ident.clone()));

        for block_item in &self.block.0 {
            match block_item {
                BlockItem::Decl(_decl) => {}
                BlockItem::Stmt(stmt) => match stmt {
                    Stmt::Return { return_value } => {
                        let reg_res = return_value.to_commands(&mut body, &mut 0);
                        body.append_commands(vec![&format!(
                            "scoreboard players operation return_value registers = {} registers",
                            reg_res
                        )]);
                    }
                },
            }
        }

        dest.append_mcfunction(body);
    }
}

impl Program {
    pub fn to_datapack(&self, pack_name: String) -> Datapack {
        let mut namespace = Namespace::new(pack_name.clone());
        self.func_def.to_mcfunction(&mut namespace);

        let mut pop_frame = Mcfunction::new("util-pop_frame".into());
        pop_frame.append_commands(vec![
            "$data remove storage memory:stack frame[$(base_index)]",
            "scoreboard players remove base_index registers 1",
            "execute store result storage memory:temp base_index int 1.0 run scoreboard players get base_index registers"
        ]);
        namespace.append_mcfunction(pop_frame);

        let mut datapck = Datapack::new(pack_name);
        datapck.append_namespace(namespace);
        datapck
    }
}

impl Exp {
    pub fn to_commands(&self, dest: &mut Mcfunction, reg_acc: &mut u32) -> String {
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
            Exp::UnaryExp(op, exp) => {
                let reg_exp = exp.to_commands(dest, reg_acc);
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
                let reg_lhs = lhs.to_commands(dest, reg_acc);
                let reg_rhs = rhs.to_commands(dest, reg_acc);
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
