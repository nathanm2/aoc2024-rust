#![allow(dead_code)]

use clap::Parser;
use std::collections::{HashMap, HashSet};
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
    let graph = build_graph(cli.file)?;
    part1(&graph);
    Ok(())
}

fn part1(graph: &Graph) {
    for (id, node) in graph.nodes.iter() {
        println!("{}", id);
    }
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
