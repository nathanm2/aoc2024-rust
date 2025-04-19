use regex::Regex;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::{Add, Mul};
use std::process;

fn main() -> Result<(), Box<(dyn Error + 'static)>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input_file>", args[0]);
        process::exit(1);
    }

    let machines = parse_claw_file(&args[1])?;
    let total: u128 = machines
        .iter()
        .map(|m| m.tokens().and_then(|(a, b)| Some(a * 3 + b)).unwrap_or(0))
        .sum();
    println!("Original Total: {}", total);

    let new_total: u128 = machines
        .iter()
        .map(|m| {
            m.tokens_alt()
                .and_then(|(a, b)| Some(a * 3 + b))
                .unwrap_or(0)
        })
        .sum();
    println!("New Total: {}", new_total);
    Ok(())
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Pos {
    x: u128,
    y: u128,
}

impl Mul<u128> for Pos {
    type Output = Pos;

    fn mul(self, f: u128) -> Pos {
        Pos {
            x: self.x * f,
            y: self.y * f,
        }
    }
}

impl Add<Pos> for Pos {
    type Output = Pos;

    fn add(self, o: Pos) -> Pos {
        Pos {
            x: self.x + o.x,
            y: self.y + o.y,
        }
    }
}

#[derive(Debug)]
struct ClawMachine {
    a: Pos,
    b: Pos,
    prize: Pos,
}

impl ClawMachine {
    fn new(a: Pos, b: Pos, prize: Pos) -> Self {
        ClawMachine { a, b, prize }
    }

    fn solve(&self, adjust: u128) -> Option<(u128, u128)> {
        let a = self.a.x as i128;
        let b = self.b.x as i128;
        let c = self.a.y as i128;
        let d = self.b.y as i128;

        let x = (adjust + self.prize.x) as i128;
        let y = (adjust + self.prize.y) as i128;

        let detr = (a * d) - (b * c);
        if detr == 0 {
            return None;
        }

        let a_press = ((d * x) - (b * y)) / detr;
        let b_press = ((a * y) - (c * x)) / detr;
        Some((a_press as u128, b_press as u128))
    }

    fn tokens(&self) -> Option<(u128, u128)> {
        if let Some((a_press, b_press)) = self.solve(0) {
            if a_press > 100 || b_press > 100 {
                None
            } else if (self.a * a_press) + (self.b * b_press) == self.prize {
                Some((a_press, b_press))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn tokens_alt(&self) -> Option<(u128, u128)> {
        let adjust = 10000000000000;
        let result = self.solve(adjust);
        let prize_alt = Pos {
            x: self.prize.x + adjust,
            y: self.prize.y + adjust,
        };
        if let Some((a_press, b_press)) = result {
            if (self.a * a_press) + (self.b * b_press) == prize_alt {
                Some((a_press, b_press))
            } else {
                None
            }
        } else {
            None
        }
    }
}

fn parse_claw_file(path: &String) -> Result<Vec<ClawMachine>, Box<dyn Error>> {
    let button_re = Regex::new(r"Button (.): X\+(\d+), Y\+(\d+)")?;
    let prize_re = Regex::new(r"Prize: X=(\d+), Y=(\d+)")?;
    let lines = BufReader::new(File::open(path)?).lines();
    let mut button_a = Pos { x: 0, y: 0 };
    let mut button_b = Pos { x: 0, y: 0 };
    let mut machines = Vec::new();

    for (num, line) in lines.enumerate() {
        let line = line?;
        match num % 4 {
            0 => button_a = parse_button(&line, num, &button_re)?,
            1 => button_b = parse_button(&line, num, &button_re)?,
            2 => {
                let prize = parse_prize(&line, num, &prize_re)?;
                machines.push(ClawMachine::new(button_a, button_b, prize));
            }
            _ => {}
        }
    }

    Ok(machines)
}

fn parse_button(line: &str, num: usize, button_re: &Regex) -> Result<Pos, Box<dyn Error>> {
    let (_, [b, x, y]) = button_re
        .captures(line)
        .ok_or_else(|| format!("{}: invalid line\n", num))?
        .extract();

    let expected = if num % 4 == 0 { "A" } else { "B" };
    if expected != b {
        return Err(Box::<dyn Error>::from(format!("{}: invalid line\n", num)));
    }

    let x = x.parse::<u128>()?;
    let y = y.parse::<u128>()?;
    Ok(Pos { x, y })
}

fn parse_prize(line: &str, num: usize, prize_re: &Regex) -> Result<Pos, Box<dyn Error>> {
    let (_, [x, y]) = prize_re
        .captures(line)
        .ok_or_else(|| format!("{}: invalid line\n", num))?
        .extract();
    let x = x.parse::<u128>()?;
    let y = y.parse::<u128>()?;
    Ok(Pos { x, y })
}
