use super::{Datapack, Mcfunction, Namespace};

pub fn mcscript_datapack() -> Datapack {
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
    let mut namespace = Namespace::new("mcscript".into());
    namespace.append_mcfunction(init);
    namespace.append_mcfunction(pop_frame);
    let mut datapack = Datapack::new("mcscript".into());
    datapack.append_namespace(namespace);
    datapack
}
