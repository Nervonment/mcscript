use super::{Datapack, Mcfunction, Namespace};

pub fn mcscript_datapack(path: String) -> Datapack {
    let mut init = Mcfunction::new("init".into());
    init.append_commands(vec![
        "scoreboard objectives add registers dummy",
        "scoreboard players set base_index registers -1",
        "execute store result storage memory:temp base_index int 1.0 run scoreboard players get base_index registers",
        "data modify storage memory:stack frame set value []",
        "data modify storage memory:temp arguments set value {}",
        "data modify storage memory:temp custom_command_arguments set value {}",
        "data modify storage memory:temp empty_str set value \"\"",
        "data modify storage memory:temp custom_command_arguments.empty_str set value \"\"",
    ]);
    let mut pop_frame = Mcfunction::new("pop_frame".into());
    pop_frame.append_commands(vec![
        "data remove storage memory:stack frame[$(base_index)]",
        "scoreboard players remove base_index registers 1",
        "execute store result storage memory:temp base_index int 1.0 run scoreboard players get base_index registers",
    ]);
    let mut mov_m_m = Mcfunction::new("mov_m_m".into());
    mov_m_m.append_command("data modify storage $(target_path) set from storage $(src_path)");
    let mut mov_m_r = Mcfunction::new("mov_m_r".into());
    mov_m_r.append_command("execute store result storage $(target_path) int 1.0 run scoreboard players get $(src_reg) registers");
    let mut mov_r_m = Mcfunction::new("mov_r_m".into());
    mov_r_m.append_command(
        "execute store result score $(target_reg) registers run data get storage $(src_path) 1.0",
    );
    let mut load_element_path = Mcfunction::new("load_element_path".into());
    load_element_path.append_command(
        "data modify storage memory:temp element_path set value \"$(array_path)[$(subscript)]\"",
    );
    let mut load_array_size = Mcfunction::new("load_array_size".into());
    load_array_size.append_command(
        "execute store result score $(target_reg) registers run data get storage $(array_path)",
    );
    let mut array_push = Mcfunction::new("array_push".into());
    array_push.append_command(
        "data modify storage $(array_path) append from storage memory:temp element",
    );
    let mut array_pop = Mcfunction::new("array_pop".into());
    array_pop.append_command(
        "data remove storage $(array_path)[-1]",
    );
    let mut array_insert = Mcfunction::new("array_insert".into());
    array_insert.append_command(
        "data modify storage $(array_path) insert $(subscript) value $(element)",
    );
    let mut array_erase = Mcfunction::new("array_erase".into());
    array_erase.append_command(
        "data remove storage $(array_path)[$(subscript)]",
    );
    let mut namespace = Namespace::new("mcscript".into());
    namespace.append_mcfunction(init);
    namespace.append_mcfunction(pop_frame);
    namespace.append_mcfunction(mov_m_m);
    namespace.append_mcfunction(mov_m_r);
    namespace.append_mcfunction(mov_r_m);
    namespace.append_mcfunction(load_element_path);
    namespace.append_mcfunction(load_array_size);
    namespace.append_mcfunction(array_push);
    namespace.append_mcfunction(array_pop);
    namespace.append_mcfunction(array_insert);
    namespace.append_mcfunction(array_erase);
    let mut datapack = Datapack::new(path);
    datapack.append_namespace(namespace);
    datapack
}
