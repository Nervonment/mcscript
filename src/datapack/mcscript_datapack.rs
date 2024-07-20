use super::{Datapack, Mcfunction, Namespace};

pub fn mcscript_datapack(path: String) -> Datapack {
    let mut init = Mcfunction::new("init".into());
    init.append_commands(vec![
        "scoreboard objectives add registers dummy",
        "scoreboard players set base_index registers -1",
        "execute store result storage memory:temp base_index int 1.0 run scoreboard players get base_index registers",
        "data modify storage memory:stack frame set value []",
        "data modify storage memory:temp arguments set value {}",
    ]);
    let mut pop_frame = Mcfunction::new("pop_frame".into());
    pop_frame.append_commands(vec![
        "$data remove storage memory:stack frame[$(base_index)]",
        "scoreboard players remove base_index registers 1",
        "execute store result storage memory:temp base_index int 1.0 run scoreboard players get base_index registers",
    ]);
    let mut mov_m_m = Mcfunction::new("mov_m_m".into());
    mov_m_m.append_command("$data modify storage $(target_path) set from storage $(src_path)");
    let mut mov_m_r = Mcfunction::new("mov_m_r".into());
    mov_m_r.append_command("$execute store result storage $(target_path) int 1.0 run scoreboard players get $(src_reg) registers");
    let mut load_element_path_src = Mcfunction::new("load_element_path_src".into());
    load_element_path_src.append_command(
        "$data modify storage memory:temp src_path set value \"$(array_path)[$(subscript)]\"",
    );
    let mut namespace = Namespace::new("mcscript".into());
    namespace.append_mcfunction(init);
    namespace.append_mcfunction(pop_frame);
    namespace.append_mcfunction(mov_m_m);
    namespace.append_mcfunction(mov_m_r);
    namespace.append_mcfunction(load_element_path_src);
    let mut datapack = Datapack::new(path);
    datapack.append_namespace(namespace);
    datapack
}
