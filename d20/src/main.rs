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

    /// Minimum cheat savings
    #[arg(short, long)]
    threshold: i32,

    /// Cheat duration
    #[arg(short, long)]
    duration: i32,
}

fn main() -> Result<(), Box<(dyn Error + 'static)>> {
    let cli = Cli::parse();
    let maze = parse_maze_file(&cli.input)?;
    let steps = maze.run();

    maze.display(Some(&steps));
    let savings = maze.cheat_savings_freq(&steps, cli.duration);
    let count = analyze_shortcuts(&savings, cli.threshold);

    println!("Original Time: {}", steps.get(&maze.end).unwrap());
    println!("Shortcut Count: {}", count);

    Ok(())
}

fn analyze_shortcuts(freq: &HashMap<i32, i32>, threshold: i32) -> i32 {
    let mut total = 0;
    let mut keys = freq
        .keys()
        .filter(|&&x| x >= threshold)
        .map(|&x| x)
        .collect::<Vec<i32>>();
    keys.sort();
    for saving in keys {
        let count = freq.get(&saving).unwrap();
        println!("{} cheats that save {} picoseconds", count, saving);
        total += count;
    }
    total
}

fn find_cheats(start: Vec2, steps: &HashMap<Vec2, i32>, dur: i32, savings: &mut HashMap<i32, i32>) {
    if let Some(&start_steps) = steps.get(&start) {
        for x in -dur..=dur {
            for y in -dur..=dur {
                let end = start
                    + Vec2 {
                        x: x as i32,
                        y: y as i32,
                    };
                let distance = start.distance(end);
                if distance > dur {
                    continue;
                }

                if let Some(&end_steps) = steps.get(&end) {
                    let saving = end_steps - start_steps - distance;
                    if saving > 0 {
                        *savings.entry(saving).or_insert(0) += 1;
                    }
                }
            }
        }
    }
}

struct Maze {
    maze: Vec<Vec<Space>>,
    width: i32,
    height: i32,
    start: Vec2,
    end: Vec2,
}

impl Maze {
    fn get(&self, pos: Vec2) -> Option<Space> {
        let x = pos.x;
        let y = pos.y;

        (x >= 0 && x < self.width && y >= 0 && y < self.height)
            .then(|| self.maze[y as usize][x as usize])
    }

    fn run(&self) -> HashMap<Vec2, i32> {
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

    fn next(&self, cur: Vec2, visited: &HashMap<Vec2, i32>) -> Option<Vec2> {
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

    fn display(&self, visited: Option<&HashMap<Vec2, i32>>) {
        let mut s = String::with_capacity((self.width + 1 * self.height) as usize);

        for y in 0..self.height {
            for x in 0..self.width {
                let v = Vec2 { x, y };

                let mut ch = self.maze[y as usize][x as usize].into();

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

    fn cheat_savings_freq(
        &self,
        steps: &HashMap<Vec2, i32>,
        cheat_duration: i32,
    ) -> HashMap<i32, i32> {
        let mut savings = HashMap::new();

        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                let pos = Vec2 {
                    x: x as i32,
                    y: y as i32,
                };
                find_cheats(pos, steps, cheat_duration, &mut savings);
            }
        }

        savings
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

impl Vec2 {
    fn distance(&self, other: Vec2) -> i32 {
        let tmp = *self - other;
        tmp.x.abs() + tmp.y.abs()
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

    let width = maze[0].len() as i32;
    let height = maze.len() as i32;

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
