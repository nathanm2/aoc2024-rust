use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    let path = env::args().nth(1).unwrap();
    let m = Matrix::new(&path);
    let (x, y) = m.dimensions();
    let mut count = 0;
    for j in 0..y {
        for i in 0..x {
            count += xmas_search(&m, Point { x: i, y: j });
        }
    }
    println!("Count: {}", count);
}

fn xmas_search(m: &Matrix, point: Point) -> usize {
    if m.get(point) == 'A' {
        let mut v1 = 0;
        let mut v2 = 0;

        v1 |= ms_search(m, point, Dir::NE);
        v1 |= ms_search(m, point, Dir::SW);

        v2 |= ms_search(m, point, Dir::SE);
        v2 |= ms_search(m, point, Dir::NW);

        if v1 == 3 && v2 == 3 { 1 } else { 0 }
    } else {
        0
    }
}

fn ms_search(m: &Matrix, point: Point, dir: Dir) -> u8 {
    match m.get_target(point, dir, 1) {
        Some(x) if x == 'M' => 0x1,
        Some(x) if x == 'S' => 0x2,
        _ => 0x0,
    }
}

#[derive(Clone, Copy)]
enum Dir {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

#[derive(Clone, Copy)]
struct Point {
    x: usize,
    y: usize,
}

struct Matrix {
    max_x: usize,
    max_y: usize,
    m: Vec<Vec<char>>,
}

impl Matrix {
    fn new(path: &str) -> Self {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        let mut matrix = Vec::<Vec<char>>::new();
        for line in reader.lines() {
            let line = line.unwrap();
            let row = line.chars().collect();
            matrix.push(row);
        }
        Matrix {
            max_x: matrix[0].len(),
            max_y: matrix.len(),
            m: matrix,
        }
    }

    fn dimensions(&self) -> (usize, usize) {
        (self.max_x, self.max_y)
    }

    fn target(&self, point: Point, dir: Dir, len: usize) -> Option<Point> {
        let x = point.x as isize;
        let y = point.y as isize;
        let l = len as isize;
        let (next_x, next_y) = match dir {
            Dir::N => (x, y - l),
            Dir::NE => (x + l, y - l),
            Dir::E => (x + l, y),
            Dir::SE => (x + l, y + l),
            Dir::S => (x, y + l),
            Dir::SW => (x - l, y + l),
            Dir::W => (x - l, y),
            Dir::NW => (x - l, y - l),
        };

        if next_x < 0
            || next_x >= self.max_x as isize
            || next_y < 0
            || next_y >= self.max_y as isize
        {
            None
        } else {
            Some(Point {
                x: next_x as usize,
                y: next_y as usize,
            })
        }
    }

    fn get_target(&self, point: Point, dir: Dir, len: usize) -> Option<char> {
        self.target(point, dir, len).map(|p| self.get(p))
    }

    fn get(&self, point: Point) -> char {
        self.m[point.y][point.x]
    }
}
