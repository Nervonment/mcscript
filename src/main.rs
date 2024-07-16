use std::{env::args, fs::read_to_string, io::Result};

use lalrpop_util::lalrpop_mod;
use mcscript::{datapack, generator::Generator};

lalrpop_mod!(parser);

fn main() -> Result<()> {
    let mut args = args();
    args.next();
    let input = read_to_string(args.next().unwrap())?;
    let ast = parser::ProgramParser::new().parse(&input).unwrap();
    let mode = args.next().unwrap();
    if mode == "-o" {
        let output = args.next().unwrap();
        let datapack = Generator::new(output).generate(ast);
        datapack.write_to_file()?;
        datapack::mcscript_datapack::mcscript_datapack().write_to_file()?;
    } else if mode == "--show-ast" {
        println!("{:#?}", ast);
    }
    return Ok(());
}
