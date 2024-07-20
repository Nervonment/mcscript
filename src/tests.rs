use std::{
    io::{stdout, Result},
    process::Command,
};

use crossterm::{
    style::{Print, PrintStyledContent, Stylize},
    ExecutableCommand,
};

enum RT {
    Memory,
    Regsiter,
}

fn test_one(mcfunction: &str, result_type: RT, expected_result: &str) -> Result<()> {
    stdout()
        .execute(Print("running "))?
        .execute(PrintStyledContent(format!("tests:{}", mcfunction).blue()))?;
    if cfg!(target_os = "windows") {
        Command::new("mcrcon.exe")
            .args(["-p", "123", &format!("function tests:{}", mcfunction)])
            .output()
    } else {
        Command::new("mcrcon")
            .args(["-p", "123", &format!("function tests:{}", mcfunction)])
            .output()
    }?;

    let command = match result_type {
        RT::Regsiter => "scoreboard players get return_value registers",
        RT::Memory => "data get storage memory:temp return_object",
    };
    let output = if cfg!(target_os = "windows") {
        Command::new("mcrcon.exe")
            .args(["-p", "123", command])
            .output()
    } else {
        Command::new("mcrcon").args(["-p", "123", command]).output()
    };

    let result = match result_type {
        RT::Regsiter => String::from_utf8(output.unwrap().stdout[17..].to_owned())
            .unwrap()
            .split(' ')
            .collect::<Vec<_>>()[0]
            .to_owned(),
        RT::Memory => String::from_utf8(output.unwrap().stdout[48..].to_owned()).unwrap(),
    };
    let result = result.trim();

    stdout()
        .execute(Print(", "))?
        .execute(PrintStyledContent(
            format!("{}", expected_result).cyan().bold(),
        ))?
        .execute(Print(" <-> "))?
        .execute(PrintStyledContent(format!("{}", result).cyan().bold()))?
        .execute(Print(", "))?
        .execute(if result == expected_result {
            PrintStyledContent(format!("correct").green())
        } else {
            PrintStyledContent(format!("wrong").red())
        })?;
    println!();

    Ok(())
}

#[test]
fn tests() -> Result<()> {
    Command::new("cargo")
        .args([
            "run",
            "--",
            "example/tests.mcs",
            "example/test_utils.mcs",
            "-o",
            "test_server/world/datapacks/my_datapack",
        ])
        .output()?;

    if cfg!(target_os = "windows") {
        Command::new("mcrcon.exe")
            .args(["-p", "123", "reload", "function mcscript:init"])
            .output()
    } else {
        Command::new("mcrcon")
            .args(["-p", "123", "reload", "function mcscript:init"])
            .output()
    }?;

    let tests = [
        ("test1", RT::Regsiter, "1"),
        ("test2", RT::Memory, "[1, 3, 6, 10, 15, 21, 28, 36, 45, 55]"),
        ("test3", RT::Memory, "[1, 1, 2, 3, 5, 8, 13, 21, 34, 55]"),
        ("test4", RT::Regsiter, "102334155"),
    ];
    for test in tests {
        test_one(test.0, test.1, test.2)?;
    }

    Ok(())
}
