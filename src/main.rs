use std::{env::args, fs::read_to_string, io::Result};

use lalrpop_util::lalrpop_mod;
use mcscript::generator::Generator;

lalrpop_mod!(parser);

fn main() -> Result<()> {
    let mut args = args();
    args.next();
    let input = read_to_string(args.next().unwrap())?;
    let ast = parser::ProgramParser::new().parse(&input).unwrap();
    println!("{:#?}", ast);
    let datapack = Generator::new("my_datapack".into()).generate(ast);
    datapack.write_to_file()?;
    return Ok(());
}
