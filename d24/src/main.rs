#![allow(dead_code)]

use clap::Parser;
use regex::Regex;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Parser)]
struct Cli {
    /// Input file
    #[arg(short, long)]
    file: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    parse_file(cli.file)?;

    Ok(())
}

fn parse_file(path: String) -> Result<(), Box<dyn Error>> {
    let wire_re = Regex::new(r"(.*): (\d)")?;
    let comp_re = Regex::new(r"(\w+) (\w+) (\w+) -> (\w+)")?;
    let mut wire_mode = true;
    for line in BufReader::new(File::open(path)?).lines() {
        let line = line?;
        if line.is_empty() {
            wire_mode = false;
        } else if wire_mode {
            let cap = wire_re.captures(&line).ok_or_else(|| "Invalid input")?;
            println!("{} {}", &cap[1], &cap[2]);
        } else {
            let cap = comp_re.captures(&line).ok_or_else(|| "Invalid input")?;
            println!("{} {} {} = {}", &cap[1], &cap[2], &cap[3], &cap[4]);
        }
    }
    Ok(())
}
