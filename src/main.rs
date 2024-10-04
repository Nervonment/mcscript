use std::{
    collections::HashMap,
    fs::read_to_string,
    io::{stdout, Result},
    path::{Path, PathBuf},
};

use crossterm::{
    style::{PrintStyledContent, Stylize},
    ExecutableCommand,
};
use mcsc::{
    backend::{datapack, generator::Generator},
    error::{handle_parse_error, handle_semantic_error},
    frontend::{self},
};

use clap::Parser;

#[derive(Parser)]
#[command(author = "https://github.com/Nervonment")]
#[command(version = "0.1.0")]
#[command(about = "MCScript Compiler", long_about = None)]
struct CLI {
    /// Specify output datapack directory
    #[arg(short)]
    output_dir: String,

    /// Input source files
    files: Vec<String>,
}

fn main() -> Result<()> {
    let cli = CLI::parse();

    let mut compile_units = vec![];
    let mut input_files = HashMap::<String, (PathBuf, String)>::new();

    for file_name in cli.files {
        let path = Path::new(&file_name);
        stdout().execute(PrintStyledContent("   Compiling ".green().bold()))?;
        println!("{}", path.as_os_str().to_string_lossy());

        let file_name_no_extension = path.file_stem().unwrap().to_owned().into_string().unwrap();
        let input = match read_to_string(file_name.clone()) {
            Ok(res) => res,
            Err(err) => {
                stdout().execute(PrintStyledContent("error".red().bold()))?;
                println!(": {}", err);
                return Ok(());
            }
        };
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
    let output = cli.output_dir;
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
            stdout().execute(PrintStyledContent("    Finished".green().bold()))?;
        }
        Err((file_name_no_extension, err)) => {
            let (file_path, content) = &input_files[&file_name_no_extension];
            handle_semantic_error(file_path, content, &err)?;
        }
    }
    Ok(())
}
