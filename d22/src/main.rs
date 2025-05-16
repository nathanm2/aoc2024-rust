#![allow(dead_code)]

use aoc::read_ints;
use clap::Parser;
use std::collections::HashMap;
use std::error::Error;

#[derive(Parser)]
struct Cli {
    /// Input file
    #[arg(short, long)]
    file: String,
}

fn main() -> Result<(), Box<(dyn Error + 'static)>> {
    let cli = Cli::parse();
    let seeds = read_ints::<u64>(cli.file)?;

    let sum = pseudo_sum(&seeds, 2000);
    println!("Part 1: {}", sum);

    let (delta, sum) = find_max(&seeds);
    println!("Part 2: {} ({:?})", sum, delta);

    Ok(())
}

#[derive(Debug, Hash, Default, PartialEq, Eq, Clone)]
struct Deltas {
    values: [i8; 4],
}

impl Deltas {
    fn push(&mut self, value: i8) {
        self.values.rotate_left(1);
        self.values[3] = value;
    }
}

fn pseudo_sum(seeds: &Vec<u64>, count: usize) -> u64 {
    seeds.iter().map(|&seed| find_secret(seed, count)).sum()
}

fn find_max(seeds: &Vec<u64>) -> (Deltas, u64) {
    let sums: HashMap<Deltas, u64> = seeds.iter()
        .flat_map(|&seed| find_seed_max(seed, 2000))
        .fold(HashMap::new(), |mut acc, (delta, value)| {
            acc.entry(delta)
                .and_modify(|e| *e += value)
                .or_insert(value);
            acc
        });

    sums.into_iter()
        .max_by_key(|&(_, value)| value)
        .unwrap_or((Deltas::default(), 0))
}

fn find_seed_max(seed: u64, count: usize) -> HashMap<Deltas, u64> {
    let mut seed_map = HashMap::new();
    let mut secret = seed;
    let mut deltas = Deltas::default();
    let mut prior = secret % 10;

    for idx in 0..count {
        secret = next_secret(secret);
        let cur = secret % 10;
        deltas.push(cur as i8 - prior as i8);
        if idx >= 4 {
            seed_map.entry(deltas.clone()).or_insert(cur);
        }
        prior = cur;
    }
    seed_map
}

fn find_secret(start: u64, count: usize) -> u64 {
    (0..count).fold(start, |secret, _| next_secret(secret))
}

fn next_secret(secret: u64) -> u64 {
    let step1 = prune(mix(secret * 64, secret));
    let step2 = prune(mix(step1 / 32, step1));
    prune(mix(step2 * 2048, step2))
}

fn mix(value: u64, secret: u64) -> u64 {
    value ^ secret
}

fn prune(value: u64) -> u64 {
    value % 16777216
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mix() {
        assert_eq!(mix(15, 42), 37);
    }

    #[test]
    fn test_prune() {
        assert_eq!(prune(100000000), 16113920);
    }

    #[test]
    fn test_next_secret() {
        let mut secret = 123;
        let expected = vec![
            15887950, 16495136, 527345, 704524, 1553684, 12683156, 11100544, 12249484, 7753432,
            5908254,
        ];

        for idx in 0..10 {
            secret = next_secret(secret);
            assert_eq!(secret, expected[idx]);
        }
    }
}
