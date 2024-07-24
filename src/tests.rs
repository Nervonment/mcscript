use std::{
    io::{stdout, Result},
    process::Command,
};

use crossterm::{
    style::{Print, PrintStyledContent, Stylize},
    ExecutableCommand,
};

fn test_one(mcfunction: &str, expected_result: &str) -> Result<bool> {
    stdout()
        .execute(Print("running "))?
        .execute(PrintStyledContent(format!("tests:{}", mcfunction).blue()))?;
    if cfg!(target_os = "windows") {
        Command::new("mcrcon.exe")
            .args([
                "-p",
                "123",
                "function mcscript:init",
                &format!("function tests:{}", mcfunction),
            ])
            .output()
    } else {
        Command::new("mcrcon")
            .args([
                "-p",
                "123",
                "function mcscript:init",
                &format!("function tests:{}", mcfunction),
            ])
            .output()
    }?;

    let command = "data get storage memory:temp return_value";
    let output = if cfg!(target_os = "windows") {
        Command::new("mcrcon.exe")
            .args(["-p", "123", command])
            .output()
    } else {
        Command::new("mcrcon").args(["-p", "123", command]).output()
    };

    let result = String::from_utf8(output.unwrap().stdout[48..].to_owned()).unwrap();
    let result = result.trim();

    let pass = result == expected_result;
    stdout()
        .execute(Print(", "))?
        .execute(PrintStyledContent(
            format!("{}", expected_result).cyan().bold(),
        ))?
        .execute(Print(" <-> "))?
        .execute(PrintStyledContent(format!("{}", result).cyan().bold()))?
        .execute(Print(", "))?
        .execute(if pass {
            PrintStyledContent(format!("pass").green())
        } else {
            PrintStyledContent(format!("fail").red())
        })?;
    println!();

    Ok(pass)
}

#[test]
fn tests() -> Result<()> {
    println!(
        "{:?}",
        Command::new("cargo")
            .args([
                "run",
                "--",
                "example/tests.mcs",
                "example/test_utils.mcs",
                "-o",
                "test_server/world/datapacks/my_datapack",
            ])
            .output()
    );

    if cfg!(target_os = "windows") {
        Command::new("mcrcon.exe")
            .args([
                "-p",
                "123",
                "reload",
                "function mcscript:init",
                "function test_utils:init",
            ])
            .output()
    } else {
        Command::new("mcrcon")
            .args([
                "-p",
                "123",
                "reload",
                "function mcscript:init",
                "function test_utils:init",
            ])
            .output()
    }?;

    let mut all_pass = true;

    let tests = [
        ("test1", "1"),
        ("test2", "[1, 3, 6, 10, 15, 21, 28, 36, 45, 55]"),
        ("test3", "[1, 1, 2, 3, 5, 8, 13, 21, 34, 55]"),
        ("test4", "102334155"),
        ("var_defn_1", "8"),
        ("var_defn_2", "10"),
        ("var_defn_3", "6"),
        ("var_defn_4", "89"),
        ("arr_defn_1", "[0, 0, 0, 0, 0, 0, 0, 0, 0, 0]"),
        ("arr_defn_2", "[[0, 0, 0], [0, 0, 0]]"),
        ("arr_defn_3", "[[0, 0, 0], [0, 0, 0]]"),
        ("arr_init_list_1", "[21, 34, 55]"),
        ("arr_init_list_2", "[[2, 1], [4, 4], [5, 6, 7], []]"),
        ("arr_init_list_3", "[1, 2]"),
        ("arr_init_list_4", "14"),
        ("arr_subscript_0", "[1, 4, 4, 4]"),
        ("arr_subscript_1", "123"),
        ("arr_subscript_2", "[[[3, 3], [0]], [[4], [0]]]"),
        ("func_defn_1", "9"),
        ("arr_arg_1", "[[1, 1], [2, 3]]"),
        ("if_1", "1"),
        ("if_2", "1"),
        ("if_3", "25"),
        ("if_4", "-5"),
        ("if_5", "25"),
        ("if_6", "25"),
        ("while_if_1", "88"),
        ("while_1", "3"),
        ("while_2", "54"),
        ("while_3", "23"),
        ("break_1", "1225"),
        ("continue_1", "4900"),
        ("glob_var_1", "2"),
        ("glob_var_2", "89"),
        ("glob_var_3", "1"),
        ("glob_var_4", "114"),
        ("glob_var_5", "1919810"),
        ("glob_var_6", "[1, 2]"),
        ("glob_var_7", "[[1, 2], [0, 0]]"),
        ("glob_var_8", "100"),
        ("glob_var_9", "120"),
        ("glob_var_10", "1"),
        ("unary_op_1", "1"),
        ("unary_op_2", "-1"),
        ("binary_op_1", "14"),
        ("binary_op_2", "20"),
        ("binary_op_3", "28"),
        ("binary_op_4", "6"),
        ("binary_op_5", "4"),
        ("binary_op_6", "0"),
        ("binary_op_7", "1"),
        ("binary_op_8", "1"),
        ("sort_1", "[0, 1, 1, 1, 1, 1, 1, 4, 4, 5, 8, 9, 9]"),
        ("sort_2", "[0, 1, 1, 1, 1, 1, 1, 4, 4, 5, 8, 9, 9]"),
        ("sort_3", "[0, 1, 1, 1, 1, 1, 1, 4, 4, 5, 8, 9, 9]"),
        ("sort_4", "[0, 1, 1, 1, 1, 1, 1, 4, 4, 5, 8, 9, 9]"),
        ("sort_5", "[0, 1, 1, 1, 1, 1, 1, 4, 4, 5, 8, 9, 9]"),
        ("sort_6", "[0, 1, 1, 1, 1, 1, 1, 4, 4, 5, 8, 9, 9]"),
        ("sort_7", "[0, 1, 1, 1, 1, 1, 1, 4, 4, 5, 8, 9, 9]"),
    ];
    for test in tests {
        all_pass = all_pass && test_one(test.0, test.1)?;
    }
    assert!(all_pass);
    Ok(())
}
