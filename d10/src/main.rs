use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    let path = env::args().nth(1).unwrap();
    let (map, zeros) = parse_map_file(&path);

    let score: usize = zeros.iter().map(|&p| follow_paths(&map, p)).sum();
    println!("Score: {}", score);

    let rating: usize = zeros.iter().map(|&p| get_rating(&map, p)).sum();
    println!("Rating: {}", rating);
}

fn get_rating(map: &Map, pos: Pos) -> usize {
    let mut visited = HashSet::from([pos]);
    let mut current = HashSet::from([pos]);
    let mut height = map.height(pos).unwrap();

    while !current.is_empty() && height < 9 {
        let next = current
            .into_iter()
            .flat_map(|p| map.neighbors(p, 1))
            .filter(|&p| !visited.contains(&p))
            .collect::<HashSet<_>>();
        
        visited.extend(next.iter());
        current = next;
        height += 1;
    }
    visited.len()
}

fn follow_paths(map: &Map, start: Pos) -> usize {
    let mut current = HashSet::from([start]);
    let mut height = map.height(start).unwrap();

    while !current.is_empty() && height < 9 {
        current = current
            .into_iter()
            .flat_map(|pos| map.neighbors(pos, 1))
            .collect();
        height += 1;
    }
    current.len()
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct Pos {
    x: isize,
    y: isize,
}

struct Map {
    xmax: isize,
    ymax: isize,
    map: Vec<Vec<i8>>,
}

impl Map {
    fn height(&self, pos: Pos) -> Option<i8> {
        if pos.x >= 0 && pos.x < self.xmax && pos.y >= 0 && pos.y < self.ymax {
            Some(self.map[pos.y as usize][pos.x as usize])
        } else {
            None
        }
    }

    fn neighbors(&self, pos: Pos, height_delta: i8) -> Vec<Pos> {
        let mut neighbors = Vec::new();
        if let Some(current_height) = self.height(pos) {
            for (xdelta, ydelta) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let neighbor = Pos {
                    x: pos.x + xdelta,
                    y: pos.y + ydelta,
                };
                if let Some(height) = self.height(neighbor) {
                    if height == current_height + height_delta {
                        neighbors.push(neighbor);
                    }
                }
            }
        }
        neighbors
    }
}

fn parse_map_file(path: &String) -> (Map, Vec<Pos>) {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let mut map: Vec<Vec<i8>> = Vec::new();
    let mut zeros: Vec<Pos> = Vec::new();

    for (y, line) in reader.lines().map(|x| x.unwrap()).enumerate() {
        let mut row: Vec<i8> = Vec::new();
        for (x, height) in line.chars().filter_map(|c| c.to_digit(10)).enumerate() {
            if height == 0 {
                zeros.push(Pos {
                    x: x as isize,
                    y: y as isize,
                });
            }
            row.push(height as i8);
        }
        map.push(row);
    }

    (
        Map {
            xmax: map[0].len() as isize,
            ymax: map.len() as isize,
            map,
        },
        zeros,
    )
}
