#![allow(dead_code)]

use clap::Parser;
use fixedbitset::FixedBitSet;
use std::collections::HashMap;
use std::error::Error;
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
    let graph = build_graph(cli.file, 600)?;
    let triples = get_triples(&graph);
    let tset = names_start_with(&graph, 't');

    let t_count = triples.iter().filter(|e| e.member_of(&tset)).count();
    println!("Part 1: {}", t_count);

    Ok(())
}

fn names_start_with(graph: &Graph, ch: char) -> FixedBitSet {
    let mut result = FixedBitSet::with_capacity(graph.capacity);
    for (idx, node) in graph.nodes.iter().enumerate() {
        if node.name.starts_with(ch) {
            result.insert(idx);
        }
    }

    result
}

#[derive(Eq, PartialEq, PartialOrd, Ord)]
struct Triple {
    values: [usize; 3],
}

impl Triple {
    fn new(a: usize, b: usize, c: usize) -> Self {
        let mut tmp = [a, b, c];
        tmp.sort();
        Triple { values: tmp }
    }

    fn member_of(&self, white_list: &FixedBitSet) -> bool {
        self.values.iter().any(|&i| white_list.contains(i))
    }
}

fn get_triples(graph: &Graph) -> Vec<Triple> {
    let mut results = Vec::new();
    let mut explored = FixedBitSet::with_capacity(graph.capacity);

    for (idx, cur) in graph.nodes.iter().enumerate() {
        let mut cur_peers: FixedBitSet = cur.peers.difference(&explored).collect();
        while !cur_peers.is_clear() {
            let peer_idx = cur_peers.minimum().unwrap();
            let peer_node = &graph.nodes[peer_idx];
            for shared_peer in peer_node.peers.intersection(&cur_peers) {
                results.push(Triple::new(idx, peer_idx, shared_peer));
            }
            cur_peers.remove(peer_idx);
        }
        explored.insert(idx);
    }

    results.sort();
    results
}

#[derive(Debug)]
struct Node {
    name: String,
    peers: FixedBitSet,
}

impl Node {
    fn new(name: &str, capacity: usize) -> Self {
        Node {
            name: name.to_string(),
            peers: FixedBitSet::with_capacity(capacity),
        }
    }
}

#[derive(Debug)]
struct Graph {
    capacity: usize,
    nodes: Vec<Node>,
}

struct GraphBuilder {
    names: HashMap<String, usize>,
    nodes: Vec<Node>,
    capacity: usize,
}

impl GraphBuilder {
    fn new(capacity: usize) -> GraphBuilder {
        GraphBuilder {
            names: HashMap::new(),
            nodes: Vec::new(),
            capacity,
        }
    }

    fn build(self) -> Graph {
        Graph {
            capacity: self.capacity,
            nodes: self.nodes,
        }
    }

    fn find_index(&mut self, name: &str) -> usize {
        if let Some(&i) = self.names.get(name) {
            i
        } else {
            let i = self.nodes.len();
            self.nodes.push(Node::new(name, self.capacity));
            self.names.insert(name.to_string(), i);
            i
        }
    }

    fn add_peer(&mut self, left: &str, right: &str) {
        let left = self.find_index(left);
        let right = self.find_index(right);

        self.nodes[left].peers.insert(right);
        self.nodes[right].peers.insert(left);
    }
}

fn build_graph(path: String, capacity: usize) -> Result<Graph, Box<dyn Error>> {
    let mut builder = GraphBuilder::new(capacity);

    for line in BufReader::new(File::open(path)?).lines() {
        let line = line?;
        let (left, right) = line.split_once('-').ok_or_else(|| "Invalid line format")?;
        builder.add_peer(left, right);
    }
    Ok(builder.build())
}
