use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process;

fn main() -> Result<(), Box<(dyn std::error::Error + 'static)>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input_file> <blink count>", args[0]);
        process::exit(1);
    }

    let stones = parse_stone_file(&args[1])?;
    let blink_count = args[2].parse::<u64>()?;
    let mut memo = Memo::new();

    let total: u64 = stones.iter().map(|s| memo.get(*s, blink_count)).sum();
    println!("Total: {}", total);
    Ok(())
}

struct Memo {
    map: HashMap<(u64, u64), u64>,
}

impl Memo {
    fn new() -> Self {
        Memo {
            map: HashMap::new(),
        }
    }

    fn get(&mut self, value: u64, blink: u64) -> u64 {
        if let Some(result) = self.map.get(&(value, blink)) {
            *result
        } else {
            let next = stone_next(value);
            let result = if blink == 1 {
                next.len() as u64
            } else {
                next.iter().map(|&v| self.get(v, blink - 1)).sum::<u64>()
            };
            self.map.insert((value, blink), result);
            result
        }
    }
}

fn stone_next(value: u64) -> Vec<u64> {
    let mut result = Vec::new();
    if value == 0 {
        result.push(1);
    } else {
        let s = value.to_string();
        if s.len() % 2 == 0 {
            let pos = s.len() / 2;
            result.push(s[0..pos].parse::<u64>().unwrap());
            result.push(s[pos..].parse::<u64>().unwrap());
        } else {
            result.push(value * 2024);
        }
    }

    result
}
fn parse_stone_file(path: &str) -> Result<Vec<u64>, Box<dyn std::error::Error>> {
    let result: Vec<_> = BufReader::new(File::open(path)?)
        .lines()
        .next()
        .ok_or("File is empty")??
        .split_whitespace()
        .map(|n| {
            n.parse::<u64>()
                .map_err(|_| format!("Invalid number: {}", n))
        })
        .collect::<Result<Vec<_>, _>>()?;

    if result.is_empty() {
        return Err("File is empty".into());
    }

    Ok(result)
}
