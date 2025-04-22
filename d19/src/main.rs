#![allow(dead_code)]

use clap::Parser;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Parser)]
struct Cli {
    /// Input file
    #[arg(short, long, value_name = "INPUT")]
    input: String,
}

fn main() -> Result<(), Box<dyn Error + 'static>> {
    let cli = Cli::parse();
    let (towels, patterns) = parse_input(&cli.input)?;
    println!("Towels: {:?}", towels);
    println!("Patterns: {:?}", patterns);

    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum Stripe {
    W = 0,
    U = 1,
    B = 2,
    R = 3,
    G = 4,
}

impl TryFrom<char> for Stripe {
    type Error = &'static str;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'w' => Ok(Stripe::W),
            'u' => Ok(Stripe::U),
            'b' => Ok(Stripe::B),
            'r' => Ok(Stripe::R),
            'g' => Ok(Stripe::G),
            _ => Err("Unrecognized stripe pattern"),
        }
    }
}

#[derive(Debug)]
struct Pattern(Vec<Stripe>);

impl TryFrom<&str> for Pattern {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value
            .chars()
            .map(|ch| ch.try_into())
            .collect::<Result<Vec<Stripe>, _>>()
            .map(Pattern)
    }
}

fn parse_input(path: &String) -> Result<(Vec<Pattern>, Vec<Pattern>), Box<dyn Error>> {
    let mut lines = BufReader::new(File::open(path)?).lines();

    let towels = lines
        .next()
        .ok_or_else(|| format!("Missing towels line"))??
        .split(", ")
        .map(|t| t.try_into())
        .collect::<Result<Vec<Pattern>, _>>()?;

    if lines
        .next()
        .ok_or_else(|| format!("Missing separation line"))??
        != ""
    {
        return Err(format!("Non-empty separation line"))?;
    }

    let mut patterns = Vec::new();
    for line in lines {
        let l = line?;
        patterns.push(l.as_str().try_into()?);
    }

    Ok((towels, patterns))
}
