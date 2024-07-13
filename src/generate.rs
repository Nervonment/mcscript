use crate::{
    ast::{exp::Exp, BlockItem, FuncDef, Program, Stmt},
    datapack::{Datapack, Mcfunction, Namespace},
};

impl FuncDef {
    pub fn to_mcfunction(&self, namespace: &mut Namespace) {
        let mut entry = Mcfunction::new(self.ident.clone());
        entry.append_commands(vec![
            "scoreboard players add base_index registers 1",
            "execute store result storage memory:temp base_index int 1.0 run scoreboard players get base_index registers",
            &format!("function {}:{}-push_frame with storage memory:temp", namespace.name(), self.ident),
            "",
            &format!("function {}:{}-call_body with storage memory:temp", namespace.name(), self.ident),
            "",
            &format!("function {}:util-pop_frame with storage memory:temp", namespace.name()),
            ""
        ]);
        namespace.append_mcfunction(entry);

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
        namespace.append_mcfunction(push_frame);

        let mut call_body = Mcfunction::new(format!("{}-call_body", self.ident.clone()));
        call_body.append_command(&format!(
            "$function {}:{}-body with storage memory:stack frame[$(base_index)]\n",
            namespace.name(),
            self.ident
        ));
        namespace.append_mcfunction(call_body);


        let mut body = Mcfunction::new(format!("{}-body", self.ident.clone()));

        for block_item in &self.block.0 {
            match block_item {
                BlockItem::Decl(_decl) => {},
                BlockItem::Stmt(stmt) => {
                    match stmt {
                        Stmt::Return { return_value } => {
                            let num = match **return_value {
                                Exp::Number(num) => {
                                    num
                                }
                            };
                            body.append_commands(vec![
                                &format!("scoreboard players set return_value registers {}", num)
                            ]);
                        }
                    }
                }
            }
        }

        namespace.append_mcfunction(body);
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