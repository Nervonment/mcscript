use std::{env::args, fs::read_to_string, io::Result};

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(parser);

fn main() -> Result<()> {
    let mut args = args();
    args.next();
    let input = read_to_string(args.next().unwrap())?;
    let mut ast = parser::ProgramParser::new().parse(&input).unwrap();
    println!("{:#?}", ast);
    let datapack = ast.to_datapack("my_datapack".into());
    datapack.write_to_file()?;
    return Ok(());
}
