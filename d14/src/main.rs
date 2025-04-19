#![allow(dead_code)]

use clap::Parser;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::{Add, Mul};

#[derive(Parser)]
struct Cli {
    /// Input file
    #[arg(short, long, value_name = "INPUT")]
    input: String,

    /// Lobby width
    #[arg(short, long)]
    width: i32,

    /// Lobby height
    #[arg(long)]
    height: i32,

    /// Safety Factor after specified number of seconds
    #[arg(short, long, value_name = "SECONDS")]
    safety: Option<i32>,

    /// Easter egg hunt
    #[arg(long, value_name = "SECONDS")]
    hunt: Option<i32>,
}

fn main() -> Result<(), Box<(dyn Error + 'static)>> {
    let cli = Cli::parse();

    let robots = parse_robot_file(&cli.input)?;

    let lobby = Lobby {
        width: cli.width,
        height: cli.height,
    };

    if let Some(sec) = cli.safety {
        let mut quad = QuadSet::new();
        let mut frame = Frame::new();
        for robot in robots.iter() {
            let pos = lobby.wrap(&robot.steps(sec));
            frame.add(pos);
            quad.add(lobby.quad(&pos));
        }

        lobby.print_frame(&frame, sec);
        println!("Safety Factor: {}", quad.safety());
        return Ok(());
    }

    if let Some(sec) = cli.hunt {
        let mut min_distance: u32 = u32::MAX;
        for i in 1..=sec {
            let mut frame = Frame::new();

            for robot in robots.iter() {
                let pos = lobby.wrap(&robot.steps(i));
                frame.add(pos);
            }

            let distance = frame.distance(2);

            if min_distance > distance {
                min_distance = distance;
                lobby.print_frame(&frame, sec);
                println!("secs: {} min_distance: {}", i, distance);
            }
        }
    }

    Ok(())
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
struct Vec2 {
    x: i32,
    y: i32,
}

impl Mul<i32> for Vec2 {
    type Output = Vec2;

    fn mul(self, f: i32) -> Vec2 {
        Vec2 {
            x: self.x * f,
            y: self.y * f,
        }
    }
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

struct Frame(HashMap<Vec2, u32>);

impl Frame {
    fn new() -> Self {
        Frame(HashMap::<Vec2, u32>::new())
    }

    fn add(&mut self, v: Vec2) {
        self.0.entry(v).and_modify(|e| *e += 1).or_insert(1);
    }

    fn distance(&self, count: u32) -> u32 {
        self.0
            .keys()
            .map(|pos| self.nearest_neighbors(*pos, count))
            .sum()
    }

    fn nearest_neighbors(&self, pos: Vec2, count: u32) -> u32 {
        let mut seen = 0;
        let mut distance: u32 = 1;
        loop {
            seen += self.neighbors(pos, distance as i32);
            if seen >= count {
                return distance;
            } else {
                distance += 1;
            }
        }
    }

    fn neighbors(&self, pos: Vec2, distance: i32) -> u32 {
        let mut result = 0;
        for x in [-distance, distance] {
            for y in -distance..=distance {
                let delta = Vec2 { x, y };
                result += self.0.get(&(pos + delta)).unwrap_or(&0);
            }
        }

        for y in [-distance, distance] {
            for x in (-distance + 1)..distance {
                let delta = Vec2 { x, y };
                result += self.0.get(&(pos + delta)).unwrap_or(&0);
            }
        }

        result
    }
}

#[derive(Debug)]
enum Quad {
    NE,
    SE,
    SW,
    NW,
}

struct Lobby {
    width: i32,
    height: i32,
}

impl Lobby {
    fn wrap(&self, v: &Vec2) -> Vec2 {
        let mut new_x = v.x % self.width;
        let mut new_y = v.y % self.height;

        if new_x < 0 {
            new_x += self.width;
        }

        if new_y < 0 {
            new_y += self.height;
        }

        Vec2 { x: new_x, y: new_y }
    }

    fn print_frame(&self, frame: &Frame, steps: i32) {
        let cap = 80 + self.height * (self.width + 1);
        let mut s = String::with_capacity(cap as usize);

        for y in 0..self.height {
            for x in 0..self.width {
                match frame.0.get(&Vec2 { x, y }) {
                    None => {
                        s.push('.');
                    }
                    Some(x) if *x <= 9 => {
                        s.push_str(&x.to_string());
                    }
                    Some(_) => {
                        s.push('X');
                    }
                }
            }
            s.push('\n');
        }
        s.push('\n');

        println!("Frame: {}", steps);
        print!("{}", s);
    }

    fn quad(&self, v: &Vec2) -> Option<Quad> {
        let xdiv = self.width / 2;
        let ydiv = self.height / 2;

        if xdiv == v.x || ydiv == v.y {
            None
        } else {
            match (v.x < xdiv, v.y < ydiv) {
                (true, true) => Some(Quad::NW),
                (true, false) => Some(Quad::SW),
                (false, true) => Some(Quad::NE),
                (false, false) => Some(Quad::SE),
            }
        }
    }
}

struct QuadSet {
    values: [u32; 4],
}

impl QuadSet {
    fn new() -> Self {
        QuadSet {
            values: [0, 0, 0, 0],
        }
    }

    fn add(&mut self, quad: Option<Quad>) {
        let index = match quad {
            None => {
                return;
            }
            Some(Quad::NW) => 0,
            Some(Quad::NE) => 1,
            Some(Quad::SW) => 2,
            Some(Quad::SE) => 3,
        };
        self.values[index] += 1;
    }

    fn safety(&self) -> u32 {
        self.values[0] * self.values[1] * self.values[2] * self.values[3]
    }

    fn balance(&self) -> f64 {
        let left = (self.values[0] + self.values[2]) as f64;
        let right = (self.values[1] + self.values[3]) as f64;
        let total = left + right;

        ((left - right).abs() / total) * 100.0
    }
}

#[derive(Debug)]
struct Robot {
    start: Vec2,
    dir: Vec2,
}

impl Robot {
    fn steps(&self, secs: i32) -> Vec2 {
        self.start + self.dir * secs
    }
}
fn parse_robot_file(path: &String) -> Result<Vec<Robot>, Box<dyn Error>> {
    let robot_re = Regex::new(r"p=(-*\d+),(-*\d+) v=(-*\d+),(-*\d+)")?;

    let mut robots = Vec::new();
    let lines = BufReader::new(File::open(path)?).lines();

    for (num, line) in lines.enumerate() {
        let line = line?;
        let (_, [px, py, vx, vy]) = robot_re
            .captures(&line)
            .ok_or_else(|| format!("{}: invalid line\n", num))?
            .extract();
        let p = Vec2 {
            x: px.parse::<i32>()?,
            y: py.parse::<i32>()?,
        };
        let v = Vec2 {
            x: vx.parse::<i32>()?,
            y: vy.parse::<i32>()?,
        };
        robots.push(Robot { start: p, dir: v });
    }

    Ok(robots)
}
