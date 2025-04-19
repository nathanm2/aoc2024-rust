use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::Chars;

fn main() {
    let path = env::args().nth(1).unwrap();
    let (map, start) = parse_map(&path);
    let mut history = PosHistory::new();
    run(start.clone(), &map, &mut history);
    println!("Unique Locations: {}", history.map.len());

    let mut loop_makers = 0;
    for pos in history.map.keys() {
        if *pos == start.loc {
            continue;
        }
        if run(
            start.clone(),
            &AltMap::new(&map, *pos),
            &mut PosHistory::new(),
        ) {
            loop_makers += 1;
        }
    }
    println!("Loop Makers: {:?}", loop_makers);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    N = 1,
    E = 2,
    S = 4,
    W = 8,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
struct Location {
    x: isize,
    y: isize,
}

#[derive(Clone, PartialEq, Eq)]
struct Position {
    loc: Location,
    dir: Direction,
}

#[derive(Clone)]
struct PosHistory {
    map: HashMap<Location, u8>,
}

impl PosHistory {
    fn new() -> Self {
        PosHistory {
            map: HashMap::<Location, u8>::new(),
        }
    }

    fn insert(&mut self, pos: &Position) {
        self.map
            .entry(pos.loc)
            .and_modify(|s| *s |= pos.dir as u8)
            .or_insert(pos.dir as u8);
    }

    fn check(&self, pos: &Position) -> bool {
        if let Some(v) = self.map.get(&pos.loc) {
            if v & pos.dir as u8 != 0 {
                return true;
            }
        }
        false
    }
}

struct Guard {
    pos: Position,
}

impl Guard {
    fn new(pos: Position) -> Self {
        Guard { pos }
    }

    fn next_step(&self) -> Location {
        match self.pos.dir {
            Direction::N => Location {
                x: self.pos.loc.x,
                y: self.pos.loc.y - 1,
            },
            Direction::E => Location {
                x: self.pos.loc.x + 1,
                y: self.pos.loc.y,
            },
            Direction::S => Location {
                x: self.pos.loc.x,
                y: self.pos.loc.y + 1,
            },
            Direction::W => Location {
                x: self.pos.loc.x - 1,
                y: self.pos.loc.y,
            },
        }
    }

    fn next_direction(&self) -> Direction {
        match self.pos.dir {
            Direction::N => Direction::E,
            Direction::E => Direction::S,
            Direction::S => Direction::W,
            Direction::W => Direction::N,
        }
    }
}

fn run(start: Position, map: &dyn MapGetter, history: &mut PosHistory) -> bool {
    let mut guard = Guard::new(start);

    if history.check(&guard.pos) {
        return true;
    }

    history.insert(&guard.pos);
    loop {
        let next_loc = guard.next_step();
        match map.get(next_loc) {
            Some(MapElement::Block) => {
                guard.pos.dir = guard.next_direction();
            }
            Some(MapElement::Space) => {
                guard.pos.loc = next_loc;
            }
            _ => {
                return false;
            }
        }

        // Have we been here before?
        if history.check(&guard.pos) == true {
            return true;
        }
        history.insert(&guard.pos);
    }
}

enum MapElement {
    Space,
    Block,
    Character(Direction),
}

trait MapGetter {
    fn get(&self, loc: Location) -> Option<MapElement>;
}

struct Map {
    blocks: HashSet<Location>,
    x_max: isize,
    y_max: isize,
}

impl MapGetter for Map {
    fn get(&self, loc: Location) -> Option<MapElement> {
        if loc.x < 0 || loc.x > self.x_max || loc.y < 0 || loc.y > self.y_max {
            None
        } else {
            if let Some(_) = self.blocks.get(&loc) {
                Some(MapElement::Block)
            } else {
                Some(MapElement::Space)
            }
        }
    }
}

struct AltMap<'a> {
    map: &'a Map,
    loc: Location,
}

impl<'a> AltMap<'a> {
    fn new(map: &'a Map, loc: Location) -> Self {
        AltMap { map, loc }
    }
}

impl MapGetter for AltMap<'_> {
    fn get(&self, loc: Location) -> Option<MapElement> {
        match self.map.get(loc) {
            Some(MapElement::Space) if loc == self.loc => Some(MapElement::Block),
            x => x,
        }
    }
}

fn parse_map(path: &str) -> (Map, Position) {
    let mut start: Option<Position> = None;
    let mut blocks = HashSet::<Location>::new();
    let mut x_max = 0;
    let mut y_max = 0;
    for (y, row) in lines(path).enumerate() {
        let y = y as isize;
        y_max = y;
        for (x, elem) in Row::new(&row).enumerate() {
            let x = x as isize;
            x_max = x;
            match elem {
                MapElement::Block => {
                    blocks.insert(Location { x, y });
                }
                MapElement::Character(d) => {
                    start = Some(Position {
                        loc: Location { x, y },
                        dir: d,
                    })
                }
                _ => {}
            }
        }
    }
    (
        Map {
            blocks,
            x_max,
            y_max,
        },
        start.unwrap(),
    )
}

fn lines(path: &str) -> impl Iterator<Item = String> {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let lines = reader.lines().map(|x| x.unwrap());
    lines
}

struct Row<I: Iterator<Item = char>> {
    iter: I,
}

impl<'a> Row<Chars<'a>> {
    fn new(row: &'a str) -> Self {
        Row { iter: row.chars() }
    }
}

impl<I: Iterator<Item = char>> Iterator for Row<I> {
    type Item = MapElement;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            None => None,
            Some('.') => Some(MapElement::Space),
            Some('#') => Some(MapElement::Block),
            Some('^') => Some(MapElement::Character(Direction::N)),
            Some('>') => Some(MapElement::Character(Direction::E)),
            Some('v') => Some(MapElement::Character(Direction::S)),
            Some('<') => Some(MapElement::Character(Direction::W)),
            Some(x) => panic!("Unexpected character: {}", x),
        }
    }
}
