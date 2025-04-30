use crate::process::{process, ProcessLike};
use anyhow::{anyhow, Context, Result};
use std::io::{BufRead, Write};

pub fn confirm(question: &str, default: bool) -> Result<bool> {
    write!(process().stdout().lock(), "{question} ")?;
    let _ = std::io::stdout().flush();
    let input = read_line()?;
    let r = match &*input.to_lowercase() {
        "y" | "yes" => true,
        "" => default,
        _ => false, // "n" | "no" gets matched here.
    };
    writeln!(process().stdout().lock())?;
    Ok(r)
}

pub fn question_str(question: &str, default: &str) -> Result<String> {
    writeln!(process().stdout().lock(), "{question} [{default}]")?;
    let _ = std::io::stdout().flush();
    let input = read_line()?;
    writeln!(process().stdout().lock())?;
    if input.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(input)
    }
}

pub fn question_bool(question: &str, default: bool) -> Result<bool> {
    let default_text = if default { "(Y/n)" } else { "(y/N)" };
    writeln!(process().stdout().lock(), "{question} {default_text}")?;
    let _ = std::io::stdout().flush();
    let input = read_line()?;
    writeln!(process().stdout().lock())?;
    if input.is_empty() {
        Ok(default)
    } else {
        match &*input.to_lowercase() {
            "y" | "yes" => Ok(true),
            "n" | "no" => Ok(false),
            _ => Ok(default),
        }
    }
}

pub(crate) fn read_line() -> Result<String> {
    let stdin = process().stdin();
    let stdin = stdin.lock();
    let mut lines = stdin.lines();
    let lines = lines.next().transpose()?;
    lines
        .map_or_else(|| Err(anyhow!("no lines found from stdin")), Ok)
        .context("unable to read from stdin for confirmation")
}
