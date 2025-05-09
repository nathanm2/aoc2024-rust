#![allow(dead_code)]

use clap::Parser;
use std::error::Error;
use std::fmt;
use std::fs;
use std::ops::{Add, Sub};

#[derive(Parser)]
struct Cli {
    /// Input file
    #[arg(short, long, value_name = "INPUT")]
    file: Option<String>,

    /// Input line (aka desired output of the last keypad).
    #[arg(short, long)]
    line: Option<String>,

    /// Direction key inputs to the first robot.
    #[arg(short, long)]
    ops: Option<String>,

    /// Number of directional pads
    #[arg(short, long, default_value_t = 2)]
    dirpads: usize,

    /// Show the input string
    #[arg(short, long, default_value_t = false)]
    show: bool,
}

fn main() -> Result<(), Box<(dyn Error + 'static)>> {
    let cli = Cli::parse();
    let dirpads = cli.dirpads;

    if let Some(path) = cli.file {
        let mut sum = 0;
        for line in parse_input(&path)? {
            sum += process_line(dirpads, &line, cli.show)?;
        }
        println!("Total Complexity: {}", sum);
    }

    if let Some(output) = cli.line {
        process_line(dirpads, &output, cli.show)?;
    }

    if let Some(ops) = cli.ops {
        let ops = string_to_keys(&ops)?;
        let mut door = Doors::new(dirpads);
        let results = door.run(ops)?;
        println!("{}", keys_to_string(&results));
    }

    Ok(())
}

fn process_line(dirpads: usize, line: &String, show: bool) -> Result<usize, Box<dyn Error>> {
    let num = parse_initial_number(&line).unwrap_or(0);
    let outputs: Vec<NumKey> = string_to_keys(&line)?;
    let mut doors = Doors::new(dirpads);

    let len = if show {
        let inputs = doors.derive(outputs);
        println!("{}", keys_to_string(&inputs));
        inputs.len()
    } else {
        doors.complexity(outputs)
    };

    let complexity = len * num;
    println!("{} x {} = {}", complexity, num, complexity);
    Ok(complexity)
}

struct Doors {
    numpad: NumKey,
    dirpads: Vec<DirKey>,
}

impl Doors {
    fn new(dirpads: usize) -> Doors {
        Doors {
            numpad: NumKey::A,
            dirpads: vec![DirKey::A; dirpads],
        }
    }

    fn derive(&mut self, outputs: Vec<NumKey>) -> Vec<DirKey> {
        let mut result = Vec::new();
        for output in outputs {
            let r = gen_input(output, 1, self.numpad, &mut self.dirpads);
            result.extend(r);
            self.numpad = output;
        }

        result
    }

    fn complexity(&mut self, outputs: Vec<NumKey>) -> usize {
        let mut total = 0;
        for output in outputs {
            total += gen_complexity(output, 1, self.numpad, &mut self.dirpads);
            self.numpad = output;
        }

        total
    }

    fn run(&mut self, mut ops: Vec<DirKey>) -> Result<Vec<NumKey>, Box<(dyn Error)>> {
        for dirpad in self.dirpads.iter() {
            ops = dirpad.run(&ops)?.0;
            println!("{}", keys_to_string(&ops));
        }
        Ok(self.numpad.run(&ops)?.0)
    }
}

struct DirKeyOp {
    key: DirKey,
    count: usize,
}

fn get_plans<T: KeyPad>(start: T, end: T, count: usize) -> Vec<Vec<DirKeyOp>> {
    let delta = end.into() - start.into();
    let (xkey, ykey) = get_direction_keys(&delta);
    let (xcount, ycount) = (delta.x.abs() as usize, delta.y.abs() as usize);

    let mut results = Vec::new();
    let xvec = Vec2 { x: delta.x, y: 0 };
    let yvec = Vec2 { x: 0, y: delta.y };

    if xcount > 0 && start.mv(xvec).is_some() {
        results.push(add_path(xkey, xcount, ykey, ycount, count));
    }
    if ycount > 0 && start.mv(yvec).is_some() {
        results.push(add_path(ykey, ycount, xkey, xcount, count));
    }
    results
}

fn add_path(
    first_key: DirKey,
    first_count: usize,
    second_key: DirKey,
    second_count: usize,
    a_count: usize,
) -> Vec<DirKeyOp> {
    let mut path = vec![DirKeyOp {
        key: first_key,
        count: first_count,
    }];
    if second_count > 0 {
        path.push(DirKeyOp {
            key: second_key,
            count: second_count,
        });
    }
    path.push(DirKeyOp {
        key: DirKey::A,
        count: a_count,
    });

    path
}

fn get_direction_keys(delta: &Vec2) -> (DirKey, DirKey) {
    let xkey = if delta.x < 0 { DirKey::W } else { DirKey::E };
    let ykey = if delta.y < 0 { DirKey::N } else { DirKey::S };
    (xkey, ykey)
}

fn gen_input<T>(output: T, count: usize, start: T, parents: &mut [DirKey]) -> Vec<DirKey>
where
    T: KeyPad,
{
    let mut shortest: Option<Vec<DirKey>> = None;

    // Find the "plan" that generates the shortest input sequence:
    for plan in get_plans(start, output, count) {
        let mut ops = Vec::new();
        let mut pvec = parents.to_vec();
        let plen = parents.len();

        // Find the inputs:
        for op in plan {
            if plen != 0 {
                let o = gen_input(op.key, op.count, pvec[plen - 1], &mut pvec[0..plen - 1]);
                ops.extend(o);
                pvec[plen - 1] = op.key;
            } else {
                ops.extend(vec![op.key; op.count]);
            }
        }

        if plen == 0 {
            return ops;
        }

        if let Some(ref s) = shortest {
            if s.len() > ops.len() {
                shortest = Some(ops);
                parents.copy_from_slice(&pvec);
            }
        } else {
            shortest = Some(ops);
            parents.copy_from_slice(&pvec);
        }
    }

    shortest.unwrap()
}

fn gen_complexity<T>(output: T, count: usize, start: T, parents: &mut [DirKey]) -> usize
where
    T: KeyPad,
{
    let mut shortest = usize::MAX;

    // Find the "plan" that generates the shortest input sequence:
    for plan in get_plans(start, output, count) {
        let mut op_count = 0;
        let mut pvec = parents.to_vec();
        let plen = parents.len();

        // Find the inputs:
        for op in plan {
            if plen != 0 {
                op_count +=
                    gen_complexity(op.key, op.count, pvec[plen - 1], &mut pvec[0..plen - 1]);
                pvec[plen - 1] = op.key;
            } else {
                op_count += op.count;
            }
        }

        if plen == 0 {
            return op_count;
        }

        if shortest > op_count {
            shortest = op_count;
            parents.copy_from_slice(&pvec);
        }
    }

    shortest
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

impl Sub<Vec2> for Vec2 {
    type Output = Vec2;

    fn sub(self, o: Vec2) -> Vec2 {
        Vec2 {
            x: self.x - o.x,
            y: self.y - o.y,
        }
    }
}

impl Vec2 {
    fn dirs(&self) -> (DirKey, usize, DirKey, usize) {
        let xdir = if self.x < 0 { DirKey::W } else { DirKey::E };
        let ydir = if self.y < 0 { DirKey::N } else { DirKey::S };
        (xdir, self.x.abs() as usize, ydir, self.y.abs() as usize)
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

impl TryFrom<char> for NumKey {
    type Error = String;
    fn try_from(ch: char) -> Result<Self, Self::Error> {
        match ch {
            '0' => Ok(NumKey::N0),
            '1' => Ok(NumKey::N1),
            '2' => Ok(NumKey::N2),
            '3' => Ok(NumKey::N3),
            '4' => Ok(NumKey::N4),
            '5' => Ok(NumKey::N5),
            '6' => Ok(NumKey::N6),
            '7' => Ok(NumKey::N7),
            '8' => Ok(NumKey::N8),
            '9' => Ok(NumKey::N9),
            'A' => Ok(NumKey::A),
            _ => Err(format!("Unrecognized NumKey: {}", ch)),
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

fn string_to_keys<T: TryFrom<char>>(s: &String) -> Result<Vec<T>, T::Error> {
    s.chars().map(|c| T::try_from(c)).collect()
}

fn parse_initial_number(s: &str) -> Option<usize> {
    let end = s.chars().take_while(|c| c.is_ascii_digit()).count();
    s[..end].parse().ok()
}
