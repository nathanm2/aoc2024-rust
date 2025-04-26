#![allow(dead_code)]

use clap::Parser;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::{Add, Sub};

#[derive(Parser)]
struct Cli {
    /// Input file
    #[arg(short, long, value_name = "INPUT")]
    input: String,
}

fn main() -> Result<(), Box<(dyn Error + 'static)>> {
    let cli = Cli::parse();
    let maze = parse_maze_file(&cli.input)?;
    let steps = maze.run();

    maze.display(Some(&steps));
    let savings = maze.collect_shortcuts(&steps);
    display_savings(&savings);

    println!("Time: {}", steps.get(&maze.end).unwrap());

    Ok(())
}

fn display_savings(savings: &HashMap<Vec2, Option<usize>>) {
    let mut freq = savings_frequencies(&savings);
}

fn savings_frequencies(values: &HashMap<Vec2, Option<usize>>) -> HashMap<usize, usize> {
    let mut frequencies = HashMap::new();
    for &value in values.values().filter(|x| x.is_some()) {
        *frequencies.entry(value.unwrap()).or_insert(0) += 1;
    }
    frequencies
}

struct Maze {
    maze: Vec<Vec<Space>>,
    width: usize,
    height: usize,
    start: Vec2,
    end: Vec2,
}

impl Maze {
    fn get(&self, pos: Vec2) -> Option<Space> {
        let x = pos.x as usize;
        let y = pos.y as usize;

        (pos.x >= 0 && x < self.width && pos.y >= 0 && y < self.height).then(|| self.maze[y][x])
    }

    fn run(&self) -> HashMap<Vec2, usize> {
        let mut cur = self.start;
        let mut visited = HashMap::new();
        let mut steps = 0;

        while let Some(next) = self.next(cur, &visited) {
            visited.insert(cur, steps);
            steps += 1;
            cur = next;
        }
        visited.insert(cur, steps);

        visited
    }

    fn next(&self, cur: Vec2, visited: &HashMap<Vec2, usize>) -> Option<Vec2> {
        let mut result = None;
        for (x, y) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let next = Vec2 { x, y } + cur;

            if self
                .get(next)
                .filter(|&s| s != Space::Wall && !visited.contains_key(&next))
                .is_some()
            {
                assert!(result.is_none());
                result = Some(next);
            }
        }
        result
    }

    fn display(&self, visited: Option<&HashMap<Vec2, usize>>) {
        let mut s = String::with_capacity(self.width + 1 * self.height);

        for y in 0..self.height {
            for x in 0..self.width {
                let v = Vec2 {
                    x: x as i32,
                    y: y as i32,
                };

                let mut ch = self.maze[y][x].into();

                if let Some(m) = visited {
                    if let Some(value) = m.get(&v) {
                        let tmp = value % 10;
                        ch = tmp.to_string().chars().nth(0).unwrap();
                    }
                }
                s.push(ch);
            }
            s.push('\n');
        }
        s.push('\n');
        print!("{}", s);
    }

    fn collect_shortcuts(&self, visited: &HashMap<Vec2, usize>) -> HashMap<Vec2, Option<usize>> {
        let mut cheats = HashMap::new();

        // Check all wall positions for potential shortcuts
        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                let pos = Vec2 {
                    x: x as i32,
                    y: y as i32,
                };
                if Space::Wall == self.get(pos).unwrap() {
                    cheats.insert(pos, self.calculate_wall_savings(pos, visited));
                }
            }
        }
        cheats
    }

    fn calculate_wall_savings(
        &self,
        wall_pos: Vec2,
        visited: &HashMap<Vec2, usize>,
    ) -> Option<usize> {
        // Check both horizontal and vertical pairs of adjacent positions
        let directions = [
            (Vec2 { x: -1, y: 0 }, Vec2 { x: 1, y: 0 }), // horizontal
            (Vec2 { x: 0, y: -1 }, Vec2 { x: 0, y: 1 }), // vertical
        ];

        for (dir1, dir2) in directions {
            let pos1 = wall_pos + dir1;
            let pos2 = wall_pos + dir2;

            if let (Some(&steps1), Some(&steps2)) = (visited.get(&pos1), visited.get(&pos2)) {
                let savings = steps1.abs_diff(steps2) - 2;
                return Some(savings);
            }
        }
        None
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
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

impl Sub<Vec2> for Vec2 {
    type Output = Vec2;

    fn sub(self, o: Vec2) -> Vec2 {
        Vec2 {
            x: self.x - o.x,
            y: self.y - o.y,
        }
    }
}

impl fmt::Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{}", self.x, self.y)
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
