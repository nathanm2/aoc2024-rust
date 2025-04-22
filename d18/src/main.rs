#![allow(dead_code)]

use clap::Parser;
use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap, HashSet};
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
    let start = Vec2 { x: 0, y: 0 };
    let end = Vec2 {
        x: dim.x - 1,
        y: dim.y - 1,
    };

    let mut obset: HashSet<Vec2> = obstacles[0..cli.count].iter().map(|x| *x).collect();
    let board = Board::new(dim, &obset);
    let path = board.shortest_path(start, end)?;
    let pathset = path_set(&path);

    display(dim, &obset, &pathset);
    println!("Path Length: {}", pathset.len() - 1);

    // Part 2
    obset.clear();
    for i in 0..obstacles.len() {
        let ob = obstacles[i];
        obset.insert(ob);
        let board = Board::new(dim, &obset);
        if let Some(path) = board.shortest_path(start, end)? {
            let pathset = path_set(&Some(path));
            display(dim, &obset, &pathset);
            println!("{} : {}", i, pathset.len() - 1);
        } else {
            println!("Blocked: {}, {:?}", i, ob);
            break;
        }
    }
    Ok(())
}

struct Node {
    pos: Vec2,
    score: i32,
    distance: i32,
    parent: Option<Vec2>,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        let s = self.score + self.distance;
        let o = other.score + other.distance;
        s.cmp(&o)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        let s = self.score + self.distance;
        let o = other.score + other.distance;
        s == o
    }
}

impl Eq for Node {}

struct Board<'a> {
    dim: Vec2,
    bytes: &'a HashSet<Vec2>,
}

impl<'a> Board<'a> {
    fn new(dim: Vec2, bytes: &'a HashSet<Vec2>) -> Board<'a> {
        Board { dim, bytes }
    }

    fn shortest_path(&self, start: Vec2, end: Vec2) -> Result<Option<Vec<Vec2>>, String> {
        let mut closed = HashMap::new();
        let mut open = BinaryHeap::new();

        // Seed 'open' with the start Node:
        let start = Node {
            pos: start,
            score: 0,
            distance: start.distance(end),
            parent: None,
        };
        open.push(Reverse(start));

        let mut end_found = false;
        while !open.is_empty() && !end_found {
            let next = open.pop().unwrap().0;

            if closed.contains_key(&next.pos) {
                continue;
            }

            end_found = next.pos == end;

            if !end_found {
                self.explore(&next, &mut open, &closed, &end);
            }
            closed.insert(next.pos, next);
        }

        if end_found {
            Ok(Some(extract_path(&closed, &end)))
        } else {
            Ok(None)
        }
    }

    fn empty_space(&self, pos: Vec2) -> bool {
        if pos.x < 0 || pos.x >= self.dim.x || pos.y < 0 || pos.y >= self.dim.y {
            false
        } else {
            !self.bytes.contains(&pos)
        }
    }

    fn explore(
        &self,
        node: &Node,
        open: &mut BinaryHeap<Reverse<Node>>,
        closed: &HashMap<Vec2, Node>,
        end: &Vec2,
    ) {
        for (x, y) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let child_pos = node.pos + Vec2 { x, y };
            if self.empty_space(child_pos) && !closed.contains_key(&child_pos) {
                let child = Node {
                    pos: child_pos,
                    score: node.score + 1,
                    distance: child_pos.distance(*end),
                    parent: Some(node.pos),
                };
                open.push(Reverse(child));
            }
        }
    }
}

fn extract_path(closed: &HashMap<Vec2, Node>, end: &Vec2) -> Vec<Vec2> {
    let mut path = Vec::new();
    let mut pos = *end;

    loop {
        let node = closed.get(&pos).unwrap();
        path.push(node.pos);
        if let Some(parent) = node.parent {
            pos = parent;
        } else {
            break;
        }
    }
    path.reverse();
    path
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

impl Vec2 {
    fn distance(&self, from: Vec2) -> i32 {
        let v = from - *self;
        v.x.abs() + v.y.abs()
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
