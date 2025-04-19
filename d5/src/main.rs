use std::collections::HashMap;
use std::env;
use std::fmt::Debug;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    let path = env::args().nth(1).unwrap();
    run(&path);
}

fn run(path: &str) {
    let mut parser = Parser::from_path(path);
    let mut depmap = HashMap::<u16, u128>::new();
    let mut sum = 0;

    for dep in parser.deps() {
        depmap
            .entry(dep.before)
            .and_modify(|m| *m |= 1 << dep.after)
            .or_insert(1 << dep.after);
    }

    for mut updates in parser.updates() {
        if let Some((a, b)) = order_violation(&updates, &depmap) {
            updates.swap(a, b);
            while let Some((a, b)) = order_violation(&updates, &depmap) {
                updates.swap(a, b);
            }
            println!("{:?} => Fixed!", updates);
            sum += updates[updates.len() / 2];
        } else {
            println!("{:?} => Pristine!", updates);
        }
    }

    println!("{}", sum);
}

fn order_violation(updates: &Vec<u16>, depmap: &HashMap<u16, u128>) -> Option<(usize, usize)> {
    let mut pages_bitmap: u128 = 0;

    for (index, page) in updates.iter().enumerate() {
        let conflicts = *depmap.get(page).unwrap_or(&0) & pages_bitmap;
        if conflicts != 0 {
            let other_page = conflicts.trailing_zeros() as u16;
            let other_index = updates.iter().position(|x| *x == other_page).unwrap();
            return Some((index, other_index));
        } else {
            pages_bitmap |= 1 << page;
        }
    }

    None
}

struct Parser {
    lines: Box<dyn Iterator<Item = String>>,
}

impl Parser {
    fn from_path(path: &str) -> Self {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        let lines = reader.lines().map(|x| x.unwrap());
        Parser {
            lines: Box::new(lines.into_iter()),
        }
    }

    fn deps(&mut self) -> DepsIter {
        DepsIter { parser: self }
    }

    fn updates(&mut self) -> UpdatesIter {
        UpdatesIter { parser: self }
    }
}

#[derive(Debug)]
struct Dep {
    before: u16,
    after: u16,
}

struct DepsIter<'a> {
    parser: &'a mut Parser,
}

impl<'a> Iterator for DepsIter<'a> {
    type Item = Dep;

    fn next(&mut self) -> Option<Self::Item> {
        match self.parser.lines.next() {
            None => None,
            Some(line) if line.len() == 0 => None,
            Some(line) => {
                let mut split = line.split('|');
                let before = split.next().unwrap().parse::<u16>().unwrap();
                let after = split.next().unwrap().parse::<u16>().unwrap();
                Some(Dep { before, after })
            }
        }
    }
}

struct UpdatesIter<'a> {
    parser: &'a mut Parser,
}

impl<'a> Iterator for UpdatesIter<'a> {
    type Item = Vec<u16>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.parser.lines.next() {
            None => None,
            Some(line) => {
                let v: Vec<u16> = line.split(',').map(|x| x.parse::<u16>().unwrap()).collect();
                Some(v)
            }
        }
    }
}
