use std::{env::args, fs::read_to_string, io::Result, path::Path};

use lalrpop_util::lalrpop_mod;
use mcscript::{
    datapack::{self, Datapack},
    generator::Generator,
};

lalrpop_mod!(parser);

fn main() -> Result<()> {
    let mut args = args();
    args.next();
    let mut namespaces = vec![];
    let mut arg;
    loop {
        arg = args.next().unwrap();
        if arg == "-o" {
            break;
        }
        let path = Path::new(&arg);
        let input = read_to_string(arg.clone())?;
        let ast = parser::ProgramParser::new().parse(&input).unwrap();
        let namespace = Generator::new(path.file_stem().unwrap().to_owned().into_string().unwrap())
            .generate(ast);
        namespaces.push(namespace);
    }
    if arg == "-o" {
        let output = args.next().unwrap();
        let mut datapack = Datapack::new(output);
        for namespace in namespaces {
            datapack.append_namespace(namespace);
        }
        datapack.write_to_file()?;
        datapack::mcscript_datapack::mcscript_datapack().write_to_file()?;
    }
    return Ok(());
}
