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

    /// Key sequence
    #[arg(short, long)]
    keys: Option<String>,
}

fn main() -> Result<(), Box<(dyn Error + 'static)>> {
    let cli = Cli::parse();
    let input = parse_input(&cli.input)?;
    for line in input {
        println!("{}", line);
    }

    if let Some(v) = cli.keys {
        let doors = Doors::new();
        let ops = v
            .chars()
            .map(|c| DirKey::try_from(c))
            .collect::<Result<Vec<_>, _>>()?;
        let results = run(&doors, ops)?;
        println!("{}", keys_to_string(&results));
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
            Door::NumPad(dp) => return Ok(dp.run(&ops)?.0),
        };
        println!("{}", keys_to_string(&ops));
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
                DirKey::N | DirKey::E | DirKey::S | DirKey::W => {
                    let dir = match op {
                        DirKey::N => Vec2 { x: 0, y: -1 },
                        DirKey::E => Vec2 { x: 1, y: 0 },
                        DirKey::S => Vec2 { x: 0, y: 1 },
                        DirKey::W => Vec2 { x: -1, y: 0 },
                        _ => unreachable!(),
                    };
                    cur = cur
                        .mv(dir)
                        .ok_or_else(|| format!("Bad {} move: {}", op, cur))?;
                }
                DirKey::A => output.push(cur),
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
        const NUM_KEYS: [[Option<NumKey>; 3]; 4] = [
            [Some(NumKey::N7), Some(NumKey::N8), Some(NumKey::N9)],
            [Some(NumKey::N4), Some(NumKey::N5), Some(NumKey::N6)],
            [Some(NumKey::N1), Some(NumKey::N2), Some(NumKey::N3)],
            [None, Some(NumKey::N0), Some(NumKey::A)],
        ];
        let pos = Vec2::from(*self) + delta;
        if pos.x >= 0 && pos.x < 3 && pos.y >= 0 && pos.y < 4 {
            NUM_KEYS[pos.y as usize][pos.x as usize]
        } else {
            None
        }
    }
}

impl fmt::Display for NumKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const CHARS: [char; 12] = ['7', '8', '9', '4', '5', '6', '1', '2', '3', ' ', '0', 'A'];
        write!(f, "{}", CHARS[*self as usize])
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
        const DIR_KEYS: [[Option<DirKey>; 3]; 2] = [
            [None, Some(DirKey::N), Some(DirKey::A)],
            [Some(DirKey::W), Some(DirKey::S), Some(DirKey::E)],
        ];
        let pos = Vec2::from(*self) + delta;
        if pos.x >= 0 && pos.x < 3 && pos.y >= 0 && pos.y < 2 {
            DIR_KEYS[pos.y as usize][pos.x as usize]
        } else {
            None
        }
    }
}

impl fmt::Display for DirKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const CHARS: [char; 6] = [' ', '^', 'A', '<', 'v', '>'];
        write!(f, "{}", CHARS[*self as usize])
    }
}

impl TryFrom<char> for DirKey {
    type Error = String;
    fn try_from(ch: char) -> Result<Self, Self::Error> {
        match ch {
            '^' => Ok(DirKey::N),
            '>' => Ok(DirKey::E),
            'v' => Ok(DirKey::S),
            '<' => Ok(DirKey::W),
            'A' => Ok(DirKey::A),
            _ => Err(format!("Unrecognized DirKey: {}", ch)),
        }
    }
}

fn parse_input(path: &String) -> Result<Vec<String>, Box<dyn Error>> {
    let tmp = fs::read_to_string(path)?;
    Ok(tmp.trim().split("\n").map(|s| s.into()).collect())
}

fn keys_to_string<T: fmt::Display>(keys: &[T]) -> String {
    keys.iter().map(|k| k.to_string()).collect()
}
