use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Lines};

#[cfg(test)]
mod tests;

fn main() {
    if let Some(arg) = env::args().nth(1) {
        run(&arg);
    } else {
        usage();
    }
}

struct SubVec {
    v: Vec<i32>,
    skip: usize,
}

impl SubVec {
    fn new(v: Vec<i32>) -> Self {
        SubVec { v, skip: 0 }
    }
}

impl Iterator for SubVec {
    type Item = Vec<i32>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.skip == self.v.len() {
            None
        } else {
            let mut sv = Vec::with_capacity(self.v.len() - 1);
            sv.extend_from_slice(&self.v[0..self.skip]);
            sv.extend_from_slice(&self.v[self.skip + 1..]);
            self.skip += 1;
            Some(sv)
        }
    }
}

struct Reports {
    lines: Lines<BufReader<File>>,
}

impl Reports {
    fn new(path: &str) -> Self {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        Reports {
            lines: reader.lines(),
        }
    }
}

impl Iterator for Reports {
    type Item = Vec<i32>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(line) = self.lines.next() {
            let line = line.unwrap();
            let v: Vec<i32> = line
                .split_ascii_whitespace()
                .map(|x| x.parse::<i32>().unwrap())
                .collect();
            Some(v)
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Rate {
    Increasing,
    Decreasing,
}

fn get_rate(l: i32, r: i32) -> Result<Rate, ()> {
    match l - r {
        -3..0 => Ok(Rate::Increasing),
        0 => Err(()),
        1..4 => Ok(Rate::Decreasing),
        _ => Err(()),
    }
}
fn check_vector(v: &Vec<i32>) -> Result<Rate, ()> {
    let rate = get_rate(v[0], v[1])?;
    for i in 1..v.len() - 1 {
        if rate != get_rate(v[i], v[i + 1])? {
            return Err(());
        }
    }
    Ok(rate)
}

fn run(path: &str) {
    let mut safe = 0;
    for report in Reports::new(path) {
        for sub in SubVec::new(report) {
            if check_vector(&sub).is_ok() {
                safe += 1;
                break;
            }
        }
    }
    println!("Safe: {}", safe);
}

fn usage() {
    println!("Usage: <input>");
}
