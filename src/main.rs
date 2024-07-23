use std::{
    collections::HashMap,
    env::args,
    fs::read_to_string,
    io::Result,
    path::{Path, PathBuf},
};

use mcscript::{
    backend::{datapack, generator::Generator},
    error::{handle_parse_error, handle_semantic_error},
    frontend::{self},
};

fn main() -> Result<()> {
    let mut args = args();
    args.next();
    let mut compile_units = vec![];
    let mut input_files = HashMap::<String, (PathBuf, String)>::new();
    let mut arg;
    loop {
        arg = args.next().unwrap();
        if arg == "-o" {
            break;
        }
        let path = Path::new(&arg);
        let file_name_no_extension = path.file_stem().unwrap().to_owned().into_string().unwrap();
        let input = read_to_string(arg.clone())?;
        input_files.insert(
            file_name_no_extension.clone(),
            (path.to_owned(), input.clone()),
        );
        let parse_res = frontend::parser::CompileUnitParser::new().parse(&input);
        match parse_res {
            Ok(ast) => {
                compile_units.push((ast, file_name_no_extension));
            }
            Err(err) => {
                handle_parse_error(path, &input, &err)?;
            }
        }
    }
    if arg == "-o" {
        let output = args.next().unwrap();
        let generate_result = Generator::new(output.clone()).generate(compile_units);
        match generate_result {
            Ok(datapack) => {
                datapack.write_to_file()?;
                let path = Path::new(&output);
                let mut ancestors = path.ancestors();
                ancestors.next();
                let ancestor = ancestors.next();
                if ancestor.is_none() {
                    datapack::mcscript_datapack::mcscript_datapack("mcscript".into())
                        .write_to_file()?;
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
            Err((file_name_no_extension, err)) => {
                let (file_path, content) = &input_files[&file_name_no_extension];
                handle_semantic_error(file_path, content, &err)?;
            }
        }
    }
    Ok(())
}
