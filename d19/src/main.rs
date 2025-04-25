use clap::Parser;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Parser)]
struct Cli {
    /// Input file
    #[arg(short, long, value_name = "INPUT")]
    input: String,
}

fn main() -> Result<(), Box<dyn Error + 'static>> {
    let cli = Cli::parse();
    let (towels, patterns) = parse_input(&cli.input)?;
    let trie = build_trie(&towels);

    let mut count = 0;
    let mut total_ways = 0;
    for pattern in &patterns {
        let ways = ways(pattern, &trie);
        println!("{} => {}", pattern, ways);
        if ways > 0 {
            count += 1;
            total_ways += ways;
        }
    }

    println!("\nCount: {}", count);
    println!("Total Ways: {}", total_ways);
    Ok(())
}

fn ways(pattern: &Pattern, root: &TrieNode) -> usize {
    let len = pattern.0.len();
    let mut pos_ways = vec![0usize; len];

    for pos in (0..len).rev() {
        let count = check_pos(pattern, root, pos, &pos_ways);
        pos_ways[pos] = count;
    }

    pos_ways[0]
}

fn check_pos(pattern: &Pattern, root: &TrieNode, start: usize, pos_ways: &Vec<usize>) -> usize {
    let mut node = root;
    let len = pattern.0.len();
    let mut sum = 0;

    for pos in start..len {
        let idx = pattern.0[pos] as usize;
        if let Some(next) = &node.children[idx].as_ref() {
            if next.is_end && pos + 1 < len {
                sum += pos_ways[pos + 1];
            }
            node = next;
        } else {
            return sum;
        }
    }

    sum + if node.is_end { 1 } else { 0 }
}

#[derive(Debug, Clone, Copy)]
enum Stripe {
    W = 0,
    U = 1,
    B = 2,
    R = 3,
    G = 4,
}

impl TryFrom<char> for Stripe {
    type Error = &'static str;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'w' => Ok(Stripe::W),
            'u' => Ok(Stripe::U),
            'b' => Ok(Stripe::B),
            'r' => Ok(Stripe::R),
            'g' => Ok(Stripe::G),
            _ => Err("Unrecognized stripe pattern"),
        }
    }
}

impl fmt::Display for Stripe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ch = match self {
            Stripe::W => 'w',
            Stripe::U => 'u',
            Stripe::B => 'b',
            Stripe::R => 'r',
            Stripe::G => 'g',
        };
        write!(f, "{}", ch)
    }
}

#[derive(Debug)]
struct Pattern(Vec<Stripe>);

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for s in self.0.iter() {
            write!(f, "{}", s)?;
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
struct TrieNode {
    children: [Option<Box<TrieNode>>; 5], // One for each Stripe variant
    is_end: bool,
}

impl TrieNode {
    fn new() -> Self {
        TrieNode {
            children: Default::default(),
            is_end: false,
        }
    }
}

fn build_trie(patterns: &[Pattern]) -> TrieNode {
    let mut root = TrieNode::new();

    for pattern in patterns {
        let mut current = &mut root;
        for &stripe in &pattern.0 {
            let idx = stripe as usize;
            if current.children[idx].is_none() {
                current.children[idx] = Some(Box::new(TrieNode::new()));
            }
            current = current.children[idx].as_mut().unwrap();
        }
        current.is_end = true;
    }

    root
}

impl TryFrom<&str> for Pattern {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value
            .chars()
            .map(|ch| ch.try_into())
            .collect::<Result<Vec<Stripe>, _>>()
            .map(Pattern)
    }
}

fn parse_input(path: &String) -> Result<(Vec<Pattern>, Vec<Pattern>), Box<dyn Error>> {
    let mut lines = BufReader::new(File::open(path)?).lines();

    let towels = lines
        .next()
        .ok_or_else(|| format!("Missing towels line"))??
        .split(", ")
        .map(|t| t.try_into())
        .collect::<Result<Vec<Pattern>, _>>()?;

    if lines
        .next()
        .ok_or_else(|| format!("Missing separation line"))??
        != ""
    {
        return Err(format!("Non-empty separation line"))?;
    }

    let mut patterns = Vec::new();
    for line in lines {
        let l = line?;
        patterns.push(l.as_str().try_into()?);
    }

    Ok((towels, patterns))
}
