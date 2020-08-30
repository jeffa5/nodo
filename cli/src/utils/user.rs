use anyhow::Result;
use colored::Colorize;
use std::io::{stdin, stdout, BufRead, Write};

pub fn confirm(prompt: &str) -> Result<bool> {
    print!("{} [{}/n]: ", prompt, "Y".bold());
    stdout().lock().flush()?;

    let mut input = String::new();
    stdin().lock().read_line(&mut input)?;

    match input.to_lowercase().trim() {
        "" | "y" | "yes" => Ok(true),
        _ => Ok(false),
    }
}

pub fn input(prompt: &str, default: Option<&str>) -> Result<String> {
    print!("{}", prompt);
    if let Some(default) = default {
        print!(" [{}]", default)
    }
    print!(": ");
    stdout().lock().flush()?;

    let mut input = String::new();
    stdin().lock().read_line(&mut input)?;

    Ok(input.trim().to_string())
}

pub fn dir_name_string<S: AsRef<str>>(name: S) -> String {
    format!("{}", name.as_ref().blue().bold())
}

pub fn file_name_string<S: AsRef<str>>(name: S) -> String {
    format!("{}", name.as_ref().green().bold())
}
