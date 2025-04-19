use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::iter::zip;

#[cfg(test)]
mod tests;

fn main() {
    if let Some(arg) = env::args().nth(1) {
        let (mut v1, mut v2) = process_input(&arg);

        v1.sort();
        v2.sort();

        let delta = calculate_delta(&v1, &v2);
        println!("Delta: {}", delta);

        let sim = calculate_sim(&v1, &v2);
        println!("Simularity: {}", sim);
    } else {
        usage();
    }
}

fn process_input(path: &str) -> (Vec<i32>, Vec<i32>) {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let mut v1 = Vec::<i32>::new();
    let mut v2 = Vec::<i32>::new();

    for line in reader.lines() {
        let line = line.unwrap();
        let mut iter = line.split_ascii_whitespace();

        let a = iter.next().unwrap().parse::<i32>().unwrap();
        let b = iter.next().unwrap().parse::<i32>().unwrap();

        v1.push(a);
        v2.push(b);
    }
    (v1, v2)
}

fn calculate_delta(v1: &Vec<i32>, v2: &Vec<i32>) -> i32 {
    zip(v1, v2).map(|(a, b)| (a - b).abs()).sum()
}

fn collapse_vec(v: &Vec<i32>) -> HashMap<i32, i32> {
    let mut result = HashMap::<i32, i32>::new();
    for x in v {
        *result.entry(*x).or_default() += 1;
    }

    result
}

fn calculate_sim(v1: &Vec<i32>, v2: &Vec<i32>) -> i32 {
    let m2 = collapse_vec(v2);
    v1.into_iter().map(|v| v * m2.get(v).unwrap_or(&0)).sum()
}

fn usage() {
    println!("Usage: <input>");
}
