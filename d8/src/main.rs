use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::{Add, Mul, Sub};

fn main() {
    let path = env::args().nth(1).unwrap();
    let map = parse_map(&path);
    let mut antinodes: HashSet<Position> = HashSet::new();
    find_antinodes(&map, &mut antinodes);
    println!("Antinodes: {}", antinodes.len());
}

fn find_antinodes(map: &Map, antinodes: &mut HashSet<Position>) {
    for (ch, positions) in map.antennas.iter() {
        println!("{}: {:?}", ch, positions);
        for i in 0..positions.len() {
            for j in i + 1..positions.len() {
                let p1 = positions[i];
                let p2 = positions[j];
                find_harmonic_antinodes(map, antinodes, p1, p1 - p2)
            }
        }
    }
}

fn find_harmonic_antinodes(
    map: &Map,
    antinodes: &mut HashSet<Position>,
    p: Position,
    delta: Position,
) {
    antinodes.insert(p);

    for adden in [-1, 1] {
        let mut i = 0;
        loop {
            let t = p + delta * i;
            if map.is_valid(t) {
                antinodes.insert(t);
                i += adden;
            } else {
                break;
            }
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
struct Position {
    x: isize,
    y: isize,
}

impl Add for Position {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Position {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Position {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Position {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<isize> for Position {
    type Output = Position;

    fn mul(self, s: isize) -> Self::Output {
        Position {
            x: self.x * s,
            y: self.y * s,
        }
    }
}

struct Map {
    x_max: isize,
    y_max: isize,
    antennas: HashMap<char, Vec<Position>>,
}

impl Map {
    fn is_valid(&self, pos: Position) -> bool {
        !(pos.x < 0 || pos.x > self.x_max || pos.y < 0 || pos.y > self.y_max)
    }
}

fn parse_map(path: &str) -> Map {
    let mut x_max = 0;
    let mut y_max = 0;
    let mut antennas: HashMap<char, Vec<Position>> = HashMap::new();

    for (y, line) in lines(path).enumerate() {
        y_max = y;
        for (x, ch) in line.chars().enumerate() {
            x_max = x;
            if ch != '.' {
                antennas
                    .entry(ch)
                    .and_modify(|v| {
                        v.push(Position {
                            x: x as isize,
                            y: y as isize,
                        })
                    })
                    .or_insert(vec![Position {
                        x: x as isize,
                        y: y as isize,
                    }]);
            }
        }
    }

    Map {
        x_max: x_max as isize,
        y_max: y_max as isize,
        antennas,
    }
}

fn lines(path: &str) -> impl Iterator<Item = String> {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let lines = reader.lines().map(|x| x.unwrap());
    lines
}
