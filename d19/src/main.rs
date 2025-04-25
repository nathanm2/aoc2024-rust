#![allow(dead_code)]

use bitvector::BitVector;
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
    for pattern in &patterns {
        let pm = PatternMatch::new(pattern, &trie);
        if pm.check() {
            println!("{} => match!", pattern);
            count += 1;
        } else {
            println!("{} => no match!", pattern);
        }
    }

    println!("Count: {}", count);
    Ok(())
}

fn check_result(pattern: &Pattern, offsets: &Vec<usize>, root: &TrieNode) -> Result<(), String> {
    let mut pattern_offset = 0;
    // println!("{:?}", offsets);

    for offset in offsets {
        let towel = Pattern(
            pattern.0[pattern_offset..pattern_offset + offset]
                .iter()
                .map(|&s| s)
                .collect::<Vec<Stripe>>(),
        );

        if !single_match(&towel, root) {
            return Err(format!("Invalid towel: {}", towel));
        }

        print!("{} ", towel);
        pattern_offset += offset;
    }

    println!("");

    Ok(())
}

struct PatternMatch<'a> {
    pattern: &'a Pattern,
    root: &'a TrieNode,
}

impl<'a> PatternMatch<'a> {
    fn new(pattern: &'a Pattern, root: &'a TrieNode) -> PatternMatch<'a> {
        PatternMatch { pattern, root }
    }

    fn check(&self) -> bool {
        let len = self.pattern.0.len();
        let mut matches = BitVector::new(len);
        for pos in (0..len).rev() {
            if self.check_pos(pos, &matches) {
                matches.insert(pos);
            }
        }

        matches.contains(0)
    }

    fn check_pos(&self, start: usize, matches: &BitVector) -> bool {
        let mut cur_node = self.root;
        let len = self.pattern.0.len();

        for offset in start..len {
            let idx = self.pattern.0[offset] as usize;
            if let Some(next_node) = &cur_node.children[idx].as_ref() {
                if next_node.is_end && offset + 1 < len && matches.contains(offset + 1) {
                    return true;
                } else {
                    cur_node = next_node;
                }
            } else {
                return false;
            }
        }

        cur_node.is_end
    }
}

fn single_match(pattern: &Pattern, root: &TrieNode) -> bool {
    let mut current = root;

    for &stripe in pattern.0.iter() {
        let idx = stripe as usize;

        if let Some(next) = &current.children[idx].as_ref() {
            current = next;
        } else {
            return false;
        }
    }

    if current.is_end { true } else { false }
}

fn multi_match(pattern: &Pattern, root: &TrieNode) -> Option<Vec<usize>> {
    if let Some(mut offsets) = pm_inner(&pattern.0, root, root) {
        offsets.reverse();
        Some(offsets)
    } else {
        None
    }
}

fn pm_inner(pattern: &[Stripe], start: &TrieNode, root: &TrieNode) -> Option<Vec<usize>> {
    let mut current = start;

    for (offset, &stripe) in pattern.iter().enumerate() {
        let idx = stripe as usize;

        if let Some(next) = &current.children[idx].as_ref() {
            if next.is_end {
                if let Some(mut m) = pm_inner(&pattern[offset + 1..], next, root) {
                    let child_offset = m.pop().unwrap_or(0);
                    m.push(offset + 1 + child_offset);
                    return Some(m);
                } else if let Some(mut m) = pm_inner(&pattern[offset + 1..], root, root) {
                    m.push(offset + 1);
                    return Some(m);
                } else {
                    return None;
                }
            }
            current = next;
        } else {
            return None;
        }
    }

    if current.is_end {
        Some(Vec::new())
    } else {
        None
    }
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
