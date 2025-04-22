#![allow(dead_code)]

use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process;

fn main() -> Result<(), Box<(dyn Error + 'static)>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input_file>", args[0]);
        process::exit(1);
    }

    let map = parse_map_file(&args[1])?;
    let regions = find_regions(&map);

    let total: usize = regions.iter().map(|r| r.area() * r.perimeter).sum();
    println!("Total: {}", total);

    let bulk_total: usize = regions.iter().map(|r| r.area() * r.sides).sum();
    println!("Bulk Total: {}", bulk_total);
    Ok(())
}

fn find_regions(map: &Map) -> Vec<Region> {
    let mut unexplored: HashSet<Pos> = HashSet::new();
    let (x_max, y_max) = map.dimensions();
    for x in 0..x_max {
        for y in 0..y_max {
            unexplored.insert(Pos {
                x: x as isize,
                y: y as isize,
            });
        }
    }

    let mut regions = Vec::new();
    while unexplored.is_empty() == false {
        let start = unexplored.iter().next().unwrap();
        let (region, explored) = Region::new(map, *start);
        unexplored = unexplored.difference(&explored).map(|&x| x).collect();
        regions.push(region);
    }

    regions
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum Dir {
    North,
    East,
    South,
    West,
}

impl Dir {
    fn step(&self) -> Pos {
        let (x, y) = match self {
            Dir::North => (0, -1),
            Dir::South => (0, 1),
            Dir::East => (1, 0),
            Dir::West => (-1, 0),
        };
        Pos { x, y }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct Edge {
    dir: Dir,
    pos: Pos,
}

#[derive(Debug)]
struct Region {
    ch: char,
    area: usize,
    perimeter: usize,
    sides: usize,
}

impl Region {
    fn new(map: &Map, start: Pos) -> (Region, HashSet<Pos>) {
        let mut explored = HashSet::new();
        let mut current = HashSet::new();
        let mut next = HashSet::new();
        let mut edges = HashSet::new();

        let ch = map.get_pos(start).unwrap();
        current.insert(start);

        while current.is_empty() == false {
            for pos in current.iter() {
                examine_peers(map, *pos, ch, &mut edges, &mut next);
            }
            explored = explored.union(&current).map(|&x| x).collect();
            current = next.difference(&explored).map(|&x| x).collect();
        }

        let perimeter = edges.len();
        let sides = count_sides(edges);

        let region = Region {
            ch,
            area: explored.len(),
            perimeter,
            sides,
        };

        println!("{:?}", region);

        (region, explored)
    }

    fn area(&self) -> usize {
        self.area
    }
}

fn examine_peers(
    map: &Map,
    pos: Pos,
    ch: char,
    edges: &mut HashSet<Edge>,
    next: &mut HashSet<Pos>,
) {
    for dir in [Dir::North, Dir::East, Dir::South, Dir::West] {
        let peer_pos = pos + dir.step();
        match map.get_pos(peer_pos) {
            None => edges.insert(Edge { dir, pos }),
            Some(peer_ch) if peer_ch == ch => next.insert(peer_pos),
            Some(_) => edges.insert(Edge { dir, pos }),
        };
    }
}

fn count_sides(mut edges: HashSet<Edge>) -> usize {
    let mut sides = 0;
    while edges.is_empty() == false {
        let edge = edges.iter().next().unwrap();
        let side = explore_side(&edges, *edge);
        sides += 1;

        edges = edges.difference(&side).map(|&x| x).collect();
    }
    sides
}

fn explore_side(edges: &HashSet<Edge>, edge: Edge) -> HashSet<Edge> {
    let mut side = HashSet::new();
    side.insert(edge);

    if edge.dir == Dir::North || edge.dir == Dir::South {
        explore_half_side(edges, &mut side, edge, Pos { x: 1, y: 0 });
        explore_half_side(edges, &mut side, edge, Pos { x: -1, y: 0 });
    } else {
        explore_half_side(edges, &mut side, edge, Pos { x: 0, y: 1 });
        explore_half_side(edges, &mut side, edge, Pos { x: 0, y: -1 });
    }

    side
}

fn explore_half_side(edges: &HashSet<Edge>, side: &mut HashSet<Edge>, start: Edge, step: Pos) {
    let mut peer_pos = start.pos + step;
    while let Some(e) = edges.get(&Edge {
        dir: start.dir,
        pos: peer_pos,
    }) {
        side.insert(*e);
        peer_pos = peer_pos + step;
    }
}

struct Map {
    map: Vec<Vec<char>>,
    x_max: isize,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
struct Pos {
    x: isize,
    y: isize,
}

impl std::ops::Add<Pos> for Pos {
    type Output = Pos;

    fn add(self, rhs: Pos) -> Pos {
        Pos {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Map {
    fn in_bounds(&self, x: isize, y: isize) -> bool {
        x >= 0 && x < self.x_max && y >= 0 && y < self.map.len() as isize
    }

    fn get(&self, x: isize, y: isize) -> Option<char> {
        self.in_bounds(x, y)
            .then(|| self.map[y as usize][x as usize])
    }

    fn get_pos(&self, pos: Pos) -> Option<char> {
        self.get(pos.x, pos.y)
    }

    fn dimensions(&self) -> (usize, usize) {
        (self.x_max as usize, self.map.len())
    }
}

fn parse_map_file(path: &String) -> Result<Map, Box<dyn Error>> {
    let mut map = Vec::new();
    for line in BufReader::new(File::open(path)?).lines() {
        let mut row = Vec::new();
        for char in line?.chars() {
            row.push(char);
        }
        map.push(row);
    }

    let x_max = map[0].len() as isize;
    Ok(Map { map, x_max })
}
