#![allow(dead_code)]

use clap::Parser;
use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::Add;

#[derive(Parser)]
struct Cli {
    /// Input file
    #[arg(short, long, value_name = "INPUT")]
    input: String,
}

fn main() -> Result<(), Box<(dyn Error + 'static)>> {
    let cli = Cli::parse();
    let maze = parse_maze_file(&cli.input)?;

    let (score, visited) = maze.score(maze.start, maze.end).unwrap();
    println!("Part 1 Score: {}", score);

    let count = maze.best_paths_count(maze.end, &visited);
    println!("Part 2 Best Paths: {}", count);

    Ok(())
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Space {
    Empty,
    Wall,
    Start,
    End,
}

impl From<Space> for char {
    fn from(value: Space) -> Self {
        match value {
            Space::Empty => '.',
            Space::Wall => '#',
            Space::Start => 'S',
            Space::End => 'E',
        }
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Facing {
    N = 0,
    E = 1,
    S = 2,
    W = 3,
}

impl Facing {
    fn opposite(self: Facing) -> Facing {
        match self {
            Facing::N => Facing::S,
            Facing::E => Facing::W,
            Facing::S => Facing::N,
            Facing::W => Facing::E,
        }
    }
}

impl From<Facing> for Vec2 {
    fn from(face: Facing) -> Self {
        match face {
            Facing::N => Vec2 { x: 0, y: -1 },
            Facing::E => Vec2 { x: 1, y: 0 },
            Facing::S => Vec2 { x: 0, y: 1 },
            Facing::W => Vec2 { x: -1, y: 0 },
        }
    }
}

#[derive(Debug)]
struct Node {
    pos: Vec2,
    facing: Facing,
    score: usize,
}

impl Node {
    fn get_score(&self, dir: Facing) -> usize {
        if dir.opposite() == self.facing {
            self.score
        } else {
            self.score + 1000
        }
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl Eq for Node {}

#[derive(Debug)]
struct Maze {
    maze: Vec<Vec<Space>>,
    width: usize,
    height: usize,
    start: Vec2,
    end: Vec2,
}

type VisitedMap = HashMap<Vec2, Node>;
type Frontier = BinaryHeap<Reverse<Node>>;

struct ParentVisitor {
    pos: Vec2,
    dir: Facing,
}

fn find_parents(v: &ParentVisitor, parents: &VisitedMap, next: &mut Vec<ParentVisitor>) {
    let node = parents.get(&v.pos).unwrap();
    let score = node.get_score(v.dir);

    for face in [Facing::N, Facing::E, Facing::S, Facing::W] {
        if v.dir.opposite() == face {
            continue;
        }

        let parent_pos = v.pos + face.into();

        if let Some(peer) = parents.get(&parent_pos) {
            if peer.get_score(face) < score {
                next.push(ParentVisitor {
                    pos: parent_pos,
                    dir: face,
                })
            }
        }
    }
}

impl Maze {
    fn get(&self, pos: Vec2) -> Option<Space> {
        let x = pos.x as usize;
        let y = pos.y as usize;

        (pos.x >= 0 && x < self.width && pos.y >= 0 && y < self.height).then(|| self.maze[y][x])
    }

    fn best_paths_count(&self, end: Vec2, visited: &VisitedMap) -> usize {
        let end_node = visited.get(&end).unwrap();
        let mut sched = vec![ParentVisitor {
            pos: end_node.pos,
            dir: end_node.facing.opposite(),
        }];
        let mut seen = HashSet::new();

        while !sched.is_empty() {
            let next = sched.pop().unwrap();
            seen.insert(next.pos);
            find_parents(&next, visited, &mut sched);
        }

        self.display(&seen);
        seen.len()
    }

    fn score(&self, start: Vec2, end: Vec2) -> Option<(usize, VisitedMap)> {
        let mut visited = VisitedMap::new();
        let mut frontier = Frontier::new();
        let mut score = None;

        frontier.push(Reverse(Node {
            pos: start,
            facing: Facing::E,
            score: 0,
        }));

        while frontier.is_empty() == false {
            let node = frontier.pop().unwrap();

            if let Some(x) = score {
                if node.0.score > x {
                    break;
                }
            }

            if node.0.pos == end {
                score = Some(node.0.score);
            }

            self.explore_node(node.0, &mut visited, &mut frontier);
        }

        score.map(|s| (s, visited))
    }

    fn explore_node(&self, node: Node, visited: &mut VisitedMap, frontier: &mut Frontier) {
        if visited.contains_key(&node.pos) {
            return;
        }

        for face in [Facing::N, Facing::E, Facing::S, Facing::W] {
            if face == node.facing.opposite() {
                continue;
            }

            let peer_pos = node.pos + face.into();
            if self.get(peer_pos).unwrap() != Space::Wall {
                let peer_score = node.score + 1 + if face != node.facing { 1000 } else { 0 };
                frontier.push(Reverse(Node {
                    pos: peer_pos,
                    facing: face,
                    score: peer_score,
                }));
            }
        }

        visited.insert(node.pos, node);
    }

    fn display(&self, visited: &HashSet<Vec2>) {
        let mut s = String::with_capacity(self.width + 1 * self.height);

        for y in 0..self.height {
            for x in 0..self.width {
                let ch = if visited.contains(&Vec2 {
                    x: x as i32,
                    y: y as i32,
                }) {
                    '*'
                } else {
                    self.maze[y][x].into()
                };
                s.push(ch);
            }
            s.push('\n');
        }
        s.push('\n');
        print!("{}", s);
    }
}

fn parse_maze_file(path: &String) -> Result<Maze, Box<dyn Error>> {
    let mut maze = Vec::new();
    let mut start = Vec2 { x: 0, y: 0 };
    let mut end = Vec2 { x: 0, y: 0 };

    for (y, line) in BufReader::new(File::open(path)?).lines().enumerate() {
        let line = line?;
        let row = parse_maze_row(&line)?;
        if let Some(x) = row.iter().position(|s| *s == Space::Start) {
            start = Vec2 {
                x: x as i32,
                y: y as i32,
            };
        }
        if let Some(x) = row.iter().position(|s| *s == Space::End) {
            end = Vec2 {
                x: x as i32,
                y: y as i32,
            };
        }
        maze.push(row);
    }

    let width = maze[0].len();
    let height = maze.len();
    Ok(Maze {
        maze,
        width,
        height,
        start,
        end,
    })
}

fn parse_maze_row(line: &str) -> Result<Vec<Space>, String> {
    line.chars()
        .map(|c| match c {
            '#' => Ok(Space::Wall),
            '.' => Ok(Space::Empty),
            'S' => Ok(Space::Start),
            'E' => Ok(Space::End),
            _ => Err(format!("Unexpected board character: {}", c)),
        })
        .collect()
}
