#![allow(dead_code)]

use clap::Parser;
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::Add;

#[derive(Parser)]
struct Cli {
    /// Input file
    #[arg(short, long, value_name = "INPUT")]
    input: String,

    /// Memory space dimensions
    #[arg(short, long, value_name = "X,Y")]
    dimensions: String,

    /// Corruption count
    #[arg(short, long)]
    count: usize,
}

fn main() -> Result<(), Box<(dyn Error + 'static)>> {
    let cli = Cli::parse();
    let dim = parse_tuple(&cli.dimensions)?;
    let locs = parse_input(&cli.input, dim)?;

    let bytes: HashSet<_> = locs[0..cli.count].iter().map(|x| *x).collect();
    display(&bytes, dim);

    Ok(())
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
struct Vec2 {
    x: i32,
    y: i32,
}

impl Add<Vec2> for Vec2 {
    type Output = Vec2;

    fn add(self, o: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + o.x,
            y: self.y + o.y,
        }
    }
}

fn display(bytes: &HashSet<Vec2>, dim: Vec2) {
    let cap = (dim.x + 1) * dim.y;
    let mut map = String::with_capacity(cap as usize);
    for y in 0..dim.y {
        for x in 0..dim.x {
            let x = x as i32;
            let y = y as i32;
            let c = if bytes.contains(&Vec2 { x, y }) {
                '#'
            } else {
                '.'
            };
            map.push(c);
        }
        map.push('\n');
    }
    print!("{}", map);
}

fn parse_tuple(s: &str) -> Result<Vec2, Box<dyn Error>> {
    if let Some((x, y)) = s.split_once(',') {
        let x = x.parse::<i32>()?;
        let y = y.parse::<i32>()?;
        Ok(Vec2 { x, y })
    } else {
        Err(format!("Invalid tuple: {}", s))?
    }
}

fn parse_input(path: &str, dim: Vec2) -> Result<Vec<Vec2>, Box<dyn Error>> {
    let mut results = Vec::new();
    for line in BufReader::new(File::open(path)?).lines() {
        let line = line?;
        let v = parse_tuple(&line)?;
        if v.x >= dim.x || v.y >= dim.y {
            return Err(format!("Outside dimensions: {}", line))?;
        }
        results.push(v);
    }
    Ok(results)
}
