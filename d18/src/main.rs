#![allow(dead_code)]

use clap::Parser;
use std::collections::{BinaryHeap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::{Add, Sub};

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
    let obstacles = parse_input(&cli.input, dim)?;

    let obstacles: HashSet<Vec2> = obstacles[0..cli.count].iter().map(|x| *x).collect();
    let board = Board::new(dim, &obstacles);
    let path = board.shortest_path(
        Vec2 { x: 0, y: 0 },
        Vec2 {
            x: dim.x - 1,
            y: dim.y - 1,
        },
    )?;
    let pathset = path_set(&path);

    display(dim, &obstacles, &pathset);

    Ok(())
}

struct Node {
    pos: Vec2,
    score: i32,
    distance: i32,
    parent: Option<Vec2>,
}

struct Board<'a> {
    dim: Vec2,
    bytes: &'a HashSet<Vec2>,
}

impl<'a> Board<'a> {
    fn new(dim: Vec2, bytes: &'a HashSet<Vec2>) -> Board<'a> {
        Board { dim, bytes }
    }

    fn shortest_path(&self, start: Vec2, end: Vec2) -> Result<Option<Vec<Vec2>>, String> {
        let mut open = BinaryHeap::new();
        let start = Node {
            pos: start,
            score: 0,
            distance: (end - start).sum(),
            parent: None,
        };
        open.push(start);

        Ok(None)
    }
}

fn path_set(path: &Option<Vec<Vec2>>) -> HashSet<Vec2> {
    if let Some(x) = path {
        x.iter().map(|x| *x).collect()
    } else {
        HashSet::new()
    }
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

impl Sub<Vec2> for Vec2 {
    type Output = Vec2;

    fn sub(self, o: Vec2) -> Vec2 {
        Vec2 {
            x: self.x - o.x,
            y: self.y - o.y,
        }
    }
}

fn display(dim: Vec2, bytes: &HashSet<Vec2>, pathset: &HashSet<Vec2>) {
    let cap = (dim.x + 1) * dim.y;
    let mut map = String::with_capacity(cap as usize);
    for y in 0..dim.y {
        for x in 0..dim.x {
            let pos = Vec2 {
                x: x as i32,
                y: y as i32,
            };
            let c = if bytes.contains(&pos) {
                '#'
            } else if pathset.contains(&pos) {
                'O'
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
