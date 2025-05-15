#![allow(dead_code)]

use aoc::read_ints;
use clap::Parser;
use std::error::Error;

#[derive(Parser)]
struct Cli {
    /// Input file
    #[arg(short, long)]
    file: String,
}

fn main() -> Result<(), Box<(dyn Error + 'static)>> {
    let cli = Cli::parse();
    let values = read_ints::<u64>(cli.file)?;
    let sum: u64 = values
        .iter()
        .map(|&value| {
            let secret = find_secret(value, 2000);
            println!("{}; {}", value, secret);
            secret
        })
        .sum();
    println!("Sum: {}", sum);
    Ok(())
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
