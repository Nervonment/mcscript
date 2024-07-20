use std::{env::args, fs::read_to_string, io::Result, path::Path};

use lalrpop_util::lalrpop_mod;
use mcscript::{datapack, generator::Generator};

lalrpop_mod!(parser);

fn main() -> Result<()> {
    let mut args = args();
    args.next();
    let mut compile_units = vec![];
    let mut arg;
    loop {
        arg = args.next().unwrap();
        if arg == "-o" {
            break;
        }
        let path = Path::new(&arg);
        let input = read_to_string(arg.clone())?;
        let ast = parser::CompileUnitParser::new().parse(&input).unwrap();
        // println!("{:#?}", ast);
        compile_units.push((
            ast,
            path.file_stem().unwrap().to_owned().into_string().unwrap(),
        ));
    }
    if arg == "-o" {
        let output = args.next().unwrap();
        let datapack = Generator::new(output.clone()).generate(compile_units);
        datapack.write_to_file()?;
        let path = Path::new(&output);
        let mut ancestors = path.ancestors();
        ancestors.next();
        let ancestor = ancestors.next();
        if ancestor.is_none() {
            datapack::mcscript_datapack::mcscript_datapack("mcscript".into()).write_to_file()?;
        } else {
            datapack::mcscript_datapack::mcscript_datapack(
                ancestor
                    .unwrap()
                    .join(Path::new("mcscript"))
                    .to_str()
                    .unwrap()
                    .to_owned(),
            )
            .write_to_file()?;
        }
    }
    return Ok(());
}
