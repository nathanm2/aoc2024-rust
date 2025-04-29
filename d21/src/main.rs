#![allow(dead_code)]

use clap::Parser;
use std::error::Error;
use std::fmt;
use std::fs;
use std::ops::Add;

#[derive(Parser)]
struct Cli {
    /// Input file
    #[arg(short, long, value_name = "INPUT")]
    input: String,
}

fn main() -> Result<(), Box<(dyn Error + 'static)>> {
    let cli = Cli::parse();
    let input = parse_input(&cli.input)?;
    for line in input {
        println!("{}", line);
    }

    Ok(())
}

enum Dir {
    N,
    E,
    S,
    W,
}

impl fmt::Display for Dir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ch = match self {
            Dir::N => '^',
            Dir::E => '>',
            Dir::S => 'v',
            Dir::W => '<',
        };
        write!(f, "{}", ch)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
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

impl From<Dir> for Vec2 {
    fn from(value: Dir) -> Self {
        match value {
            Dir::N => Vec2 { x: 0, y: -1 },
            Dir::E => Vec2 { x: 1, y: 0 },
            Dir::S => Vec2 { x: 0, y: 1 },
            Dir::W => Vec2 { x: -1, y: 0 },
        }
    }
}

#[derive(Copy, Clone)]
enum NumKey {
    Num(i8),
    A,
    Gap,
}

impl fmt::Display for NumKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ch = match self {
            NumKey::Num(x) => x.to_string(),
            NumKey::A => "A".into(),
            NumKey::Gap => "G".into(),
        };
        write!(f, "{}", ch)
    }
}

const NUMPAD: [[NumKey; 3]; 4] = [
    [NumKey::Num(7), NumKey::Num(8), NumKey::Num(9)],
    [NumKey::Num(4), NumKey::Num(5), NumKey::Num(6)],
    [NumKey::Num(1), NumKey::Num(2), NumKey::Num(3)],
    [NumKey::Gap, NumKey::Num(0), NumKey::A],
];

impl From<NumKey> for Vec2 {
    fn from(value: NumKey) -> Self {
        match value {
            NumKey::Num(7) => Vec2 { x: 0, y: 0 },
            NumKey::Num(8) => Vec2 { x: 1, y: 0 },
            NumKey::Num(9) => Vec2 { x: 2, y: 0 },
            NumKey::Num(4) => Vec2 { x: 0, y: 1 },
            NumKey::Num(5) => Vec2 { x: 1, y: 1 },
            NumKey::Num(6) => Vec2 { x: 2, y: 1 },
            NumKey::Num(1) => Vec2 { x: 0, y: 2 },
            NumKey::Num(2) => Vec2 { x: 2, y: 2 },
            NumKey::Num(3) => Vec2 { x: 3, y: 2 },
            NumKey::Gap => Vec2 { x: 0, y: 3 },
            NumKey::Num(0) => Vec2 { x: 1, y: 3 },
            NumKey::A => Vec2 { x: 2, y: 3 },
            _ => panic!("Unexpected value: {}", value),
        }
    }
}

struct NumPad {
    cur: Vec2,
}

impl NumPad {
    fn new() -> Self {
        NumPad {
            cur: NumKey::A.into(),
        }
    }

    fn get(&self, pos: Vec2) -> Option<NumKey> {
        if pos.x >= 0 && pos.x < 3 && pos.y >= 0 && pos.y < 4 {
            Some(NUMPAD[pos.y as usize][pos.x as usize])
        } else {
            None
        }
    }
}

enum DirKey {
    Dir(Dir),
    A,
    Gap,
}

const DIRPAD: [[DirKey; 3]; 2] = [
    [DirKey::Gap, DirKey::Dir(Dir::N), DirKey::A],
    [
        DirKey::Dir(Dir::W),
        DirKey::Dir(Dir::S),
        DirKey::Dir(Dir::E),
    ],
];

fn parse_input(path: &String) -> Result<Vec<String>, Box<dyn Error>> {
    let tmp = fs::read_to_string(path)?;
    Ok(tmp.trim().split("\n").map(|s| s.into()).collect())
}
