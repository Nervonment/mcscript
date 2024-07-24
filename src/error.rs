use std::{
    io::{stdout, Result},
    path::Path,
};

use crossterm::{
    style::{Print, PrintStyledContent, Stylize},
    ExecutableCommand,
};
use lalrpop_util::{lexer::Token, ParseError};

use crate::backend::error::SemanticError;

pub struct Split<'a>(Vec<(&'a str, usize, usize)>);

impl<'a> Split<'a> {
    pub fn new(content: &'a str) -> Self {
        let split = content.split('\n').collect::<Vec<_>>();
        let mut split = split.iter().map(|s| (*s, 0, 0)).collect::<Vec<_>>();
        let mut loc = 0;
        for line in &mut split {
            line.1 = loc;
            loc += line.0.len() + 1;
            line.2 = loc;
        }
        Self(split)
    }

    pub fn query_loc(&self, begin: usize, end: usize) -> (usize, &str, usize, usize) {
        let mut iter = self.0.iter().enumerate();
        let mut next = iter.next();
        while next.is_some_and(|(_, (_, _, end))| *end <= begin) {
            next = iter.next();
        }
        match next {
            Some((line_num, (line, line_begin, _))) => {
                (line_num, line, begin - *line_begin, end - *line_begin)
            }
            None => unreachable!(),
        }
    }
}

pub fn show_error_message(
    file_path: &Path,
    content: &Split,
    begin: usize,
    end: usize,
    message: &str,
) -> Result<()> {
    let (line_num, line, begin, end) = content.query_loc(begin, end);
    let line_num = line_num + 1;
    let line_num = line_num.to_string();
    let space_len = line_num.len();
    let mut space = String::with_capacity(space_len);
    for _ in 0..space_len {
        space.push(' ');
    }
    let mut underline = String::with_capacity(end);
    for _ in 0..begin {
        underline.push(' ');
    }
    for _ in begin..end {
        underline.push('^');
    }

    stdout()
        .execute(PrintStyledContent("error".red()))?
        .execute(Print(": "))?
        .execute(Print(message))?
        .execute(Print("\n"))?
        .execute(Print(&space))?
        .execute(PrintStyledContent("--> ".grey()))?
        .execute(Print(format!(
            "{}:{}:{}",
            file_path.to_str().unwrap(),
            line_num,
            begin + 1
        )))?
        .execute(Print("\n"))?
        .execute(Print(&space))?
        .execute(PrintStyledContent(" |".grey()))?
        .execute(Print("\n"))?
        .execute(PrintStyledContent(format!("{} |  ", line_num).grey()))?
        .execute(Print(line))?
        .execute(Print("\n"))?
        .execute(Print(space))?
        .execute(PrintStyledContent(" |  ".grey()))?
        .execute(PrintStyledContent(underline.red()))?
        .execute(Print("\n"))?
        .execute(Print("\n"))?;

    Ok(())
}

pub fn handle_parse_error(
    file_path: &Path,
    content: &str,
    err: &ParseError<usize, Token, &str>,
) -> Result<()> {
    let content = Split::new(content);
    match err {
        ParseError::ExtraToken {
            token: (begin, token, end),
        } => {
            show_error_message(
                file_path,
                &content,
                *begin,
                *end,
                &format!("extra token: {}", token.1),
            )?;
        }
        ParseError::InvalidToken { location } => {
            show_error_message(
                file_path,
                &content,
                *location,
                location + 1,
                "invalid token",
            )?;
        }
        ParseError::UnrecognizedEof { location, expected } => {
            show_error_message(
                file_path,
                &content,
                *location,
                location + 1,
                &format!("unrecognized eof, expected: {}", expected.join(", ")),
            )?;
        }
        ParseError::UnrecognizedToken {
            token: (begin, token, end),
            expected,
        } => {
            show_error_message(
                file_path,
                &content,
                *begin,
                *end,
                &format!(
                    "unrecognized token \"{}\", expected: {}",
                    token.1,
                    expected.join(", ")
                ),
            )?;
        }
        ParseError::User { error } => {
            println!("{}", error);
        }
    }
    Ok(())
}

pub fn handle_semantic_error(file_path: &Path, content: &str, err: &SemanticError) -> Result<()> {
    let content_split = Split::new(content);
    match err {
        SemanticError::MultipleDefinition { ident, begin, end } => {
            show_error_message(
                file_path,
                &content_split,
                *begin,
                *end,
                &format!("\"{}\" is defined multiple times", ident),
            )?;
        }
        SemanticError::UndefinedIdentifier { ident, begin, end } => {
            show_error_message(
                file_path,
                &content_split,
                *begin,
                *end,
                &format!("\"{}\" is not defined", ident),
            )?;
        }
        SemanticError::TypeMismatch {
            expected_type,
            found_type: given_type,
            begin,
            end,
        } => {
            show_error_message(
                file_path,
                &content_split,
                *begin,
                *end,
                &format!(
                    "expected \"{}\" here, found \"{}\" which is of type \"{}\"",
                    expected_type,
                    &content[*begin..*end],
                    given_type
                ),
            )?;
        }
        SemanticError::ExpectedVoid {
            found_type,
            begin,
            end,
        } => {
            show_error_message(
                file_path,
                &content_split,
                *begin,
                *end,
                &format!(
                    "this is a void function and should not return a value of type \"{}\"",
                    found_type
                ),
            )?;
        }
        SemanticError::ExpectedValue {
            expected_type,
            begin,
            end,
        } => {
            show_error_message(
                file_path,
                &content_split,
                *begin,
                *end,
                &format!(
                    "this is a non-void function and should return a value of type \"{}\"",
                    expected_type
                ),
            )?;
        }
        SemanticError::IndexIntoNonArray {
            found_type,
            begin,
            end,
        } => {
            show_error_message(
                file_path,
                &content_split,
                *begin,
                *end,
                &format!("cannot index into a value of type \"{}\"", found_type),
            )?;
        }
        SemanticError::NoLoopToBreak { begin, end } => {
            show_error_message(
                file_path,
                &content_split,
                *begin,
                *end,
                "there is no loop to break",
            )?;
        }
        SemanticError::NoLoopToContinue { begin, end } => {
            show_error_message(
                file_path,
                &content_split,
                *begin,
                *end,
                "there is no loop to continue",
            )?;
        }
    }
    Ok(())
}
