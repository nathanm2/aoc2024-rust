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

#[derive(Copy, Clone)]
enum Door {
    NumPad(NumKey),
    DirPad(DirKey),
}

#[derive(Copy, Clone)]
struct Doors {
    doors: [Door; 3],
}

impl Doors {
    fn new() -> Doors {
        Doors {
            doors: [
                Door::DirPad(DirKey::A),
                Door::DirPad(DirKey::A),
                Door::NumPad(NumKey::A),
            ],
        }
    }
}

fn run(doors: &Doors, mut ops: Vec<DirKey>) -> Result<Vec<NumKey>, Box<(dyn Error)>> {
    for door in doors.doors {
        ops = match door {
            Door::DirPad(np) => np.run(&ops)?.0,
            Door::NumPad(dp) => {
                return Ok(dp.run(&ops)?.0);
            }
        }
    }
    Err(format!("No num pad"))?
}

#[derive(Copy, Clone)]
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

trait KeyPad: Sized + Into<Vec2> + fmt::Display + Copy {
    fn mv(&self, dir: Vec2) -> Option<Self>;

    fn run(self, ops: &[DirKey]) -> Result<(Vec<Self>, Self), String> {
        let mut cur = self;
        let mut output = Vec::new();

        for op in ops {
            match op {
                DirKey::N => {
                    cur = cur
                        .mv(Vec2 { x: 0, y: -1 })
                        .ok_or_else(|| format!("Bad {} move: {}", op, cur))?;
                }
                DirKey::E => {
                    cur = cur
                        .mv(Vec2 { x: 1, y: 0 })
                        .ok_or_else(|| format!("Bad {} move: {}", op, cur))?;
                }
                DirKey::S => {
                    cur = cur
                        .mv(Vec2 { x: 0, y: 1 })
                        .ok_or_else(|| format!("Bad {} move: {}", op, cur))?;
                }
                DirKey::W => {
                    cur = cur
                        .mv(Vec2 { x: -1, y: 0 })
                        .ok_or_else(|| format!("Bad {} move: {}", op, cur))?;
                }
                DirKey::A => {
                    output.push(cur);
                }
                DirKey::Gap => Err(format!("Can't run Gap"))?,
            }
        }

        Ok((output, cur))
    }
}

#[derive(Clone, Copy, Default)]
enum NumKey {
    N7 = 0,
    N8 = 1,
    N9 = 2,
    N4 = 3,
    N5 = 4,
    N6 = 5,
    N1 = 6,
    N2 = 7,
    N3 = 8,
    Gap = 9,
    N0 = 10,
    #[default]
    A = 11,
}

impl From<NumKey> for Vec2 {
    fn from(key: NumKey) -> Vec2 {
        let o = key as i32;
        Vec2 { x: o % 3, y: o / 3 }
    }
}

impl KeyPad for NumKey {
    fn mv(&self, delta: Vec2) -> Option<Self> {
        match Vec2::from(*self) + delta {
            Vec2 { x: 0, y: 0 } => Some(NumKey::N7),
            Vec2 { x: 1, y: 0 } => Some(NumKey::N8),
            Vec2 { x: 2, y: 0 } => Some(NumKey::N9),
            Vec2 { x: 0, y: 1 } => Some(NumKey::N4),
            Vec2 { x: 1, y: 1 } => Some(NumKey::N5),
            Vec2 { x: 2, y: 1 } => Some(NumKey::N6),
            Vec2 { x: 0, y: 2 } => Some(NumKey::N1),
            Vec2 { x: 1, y: 2 } => Some(NumKey::N2),
            Vec2 { x: 2, y: 2 } => Some(NumKey::N3),
            Vec2 { x: 1, y: 3 } => Some(NumKey::N0),
            Vec2 { x: 2, y: 3 } => Some(NumKey::A),
            _ => None,
        }
    }
}

impl fmt::Display for NumKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ch = match self {
            NumKey::N7 => '7',
            NumKey::N8 => '8',
            NumKey::N9 => '9',
            NumKey::N4 => '4',
            NumKey::N5 => '5',
            NumKey::N6 => '6',
            NumKey::N1 => '1',
            NumKey::N2 => '2',
            NumKey::N3 => '3',
            NumKey::Gap => ' ',
            NumKey::N0 => '0',
            NumKey::A => 'A',
        };
        write!(f, "{}", ch)
    }
}

#[derive(Clone, Copy, Default)]
enum DirKey {
    Gap = 0,
    N = 1,
    #[default]
    A = 2,
    W = 3,
    S = 4,
    E = 5,
}

impl From<DirKey> for Vec2 {
    fn from(key: DirKey) -> Vec2 {
        let o = key as i32;
        Vec2 { x: o % 3, y: o / 3 }
    }
}

impl KeyPad for DirKey {
    fn mv(&self, delta: Vec2) -> Option<Self> {
        match Vec2::from(*self) + delta {
            Vec2 { x: 1, y: 0 } => Some(DirKey::N),
            Vec2 { x: 2, y: 0 } => Some(DirKey::A),
            Vec2 { x: 0, y: 1 } => Some(DirKey::W),
            Vec2 { x: 1, y: 1 } => Some(DirKey::S),
            Vec2 { x: 2, y: 1 } => Some(DirKey::E),
            _ => None,
        }
    }
}

impl fmt::Display for DirKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ch = match self {
            DirKey::Gap => ' ',
            DirKey::N => '^',
            DirKey::A => 'A',
            DirKey::W => '<',
            DirKey::S => 'v',
            DirKey::E => '>',
        };
        write!(f, "{}", ch)
    }
}

fn parse_input(path: &String) -> Result<Vec<String>, Box<dyn Error>> {
    let tmp = fs::read_to_string(path)?;
    Ok(tmp.trim().split("\n").map(|s| s.into()).collect())
}
