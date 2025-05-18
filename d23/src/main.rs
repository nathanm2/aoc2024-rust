#![allow(dead_code)]

use clap::Parser;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Parser)]
struct Cli {
    /// Input file
    #[arg(short, long)]
    file: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let graph = build_graph(cli.file)?;
    let triples = get_triples(&graph);

    for triple in &triples {
        println!("{}", triple);
    }

    let t_count = triples
        .iter()
        .filter(|e| Triple::starts_with(e, 't'))
        .count();
    println!("Part 1: {}", t_count);

    let memberships = get_memberships(&triples);
    for membership in memberships {
        println!("{}: {}", membership.0, membership.1);
    }
    Ok(())
}

#[derive(Eq, PartialEq, PartialOrd, Ord)]
struct Triple {
    a: String,
    b: String,
    c: String,
}

impl Triple {
    fn new(a: &str, b: &str, c: &str) -> Self {
        let mut tmp = [a, b, c];
        tmp.sort();
        Triple {
            a: tmp[0].to_string(),
            b: tmp[1].to_string(),
            c: tmp[2].to_string(),
        }
    }

    fn starts_with(&self, ch: char) -> bool {
        self.a.starts_with(ch) || self.b.starts_with(ch) || self.c.starts_with(ch)
    }
}

impl fmt::Display for Triple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}-{}", self.a, self.b, self.c)
    }
}

fn get_triples(graph: &Graph) -> Vec<Triple> {
    let mut results = Vec::new();
    let mut explored = HashSet::new();

    for (name, cur) in graph.nodes.iter() {
        let mut cur_peers: HashSet<_> = cur.peers.difference(&explored).cloned().collect();
        while !cur_peers.is_empty() {
            let peer = cur_peers.iter().next().unwrap().clone();
            let peer_node = graph.nodes.get(&peer).unwrap();
            for shared_peer in peer_node.peers.intersection(&cur_peers) {
                results.push(Triple::new(name, &peer, shared_peer))
            }
            cur_peers.remove(&peer);
        }
        explored.insert(name.clone());
    }

    results.sort();
    results
}

fn get_memberships<'a>(triples: &'a Vec<Triple>) -> Vec<(&'a str, usize)> {
    let mut tmp = HashMap::<&str, usize>::new();
    for triple in triples {
        for name in [&triple.a, &triple.b, &triple.c] {
            tmp.entry(&name).and_modify(|e| *e += 1).or_insert(1);
        }
    }

    let mut results: Vec<(&str, usize)> = tmp.iter().map(|(&k, &v)| (k, v)).collect();
    results.sort_by_key(|k| k.1);
    results
}

#[derive(Debug)]
struct Node {
    peers: HashSet<String>,
}

impl Node {
    fn new() -> Self {
        Node {
            peers: HashSet::new(),
        }
    }

    fn degree(&self) -> usize {
        self.peers.len()
    }
}

#[derive(Debug)]
struct Graph {
    nodes: HashMap<String, Node>,
}

impl Graph {
    fn new() -> Self {
        Graph {
            nodes: HashMap::new(),
        }
    }

    fn add_peer(&mut self, node: &str, peer: &str) {
        self.nodes
            .entry(node.to_string())
            .or_insert_with(Node::new)
            .peers
            .insert(peer.to_string());
    }
}

fn build_graph(path: String) -> Result<Graph, Box<dyn Error>> {
    let mut graph = Graph::new();

    for line in BufReader::new(File::open(path)?).lines() {
        let line = line?;
        let (left, right) = line.split_once('-').ok_or_else(|| "Invalid line format")?;
        graph.add_peer(left, right);
        graph.add_peer(right, left);
    }
    Ok(graph)
}
