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
    let mut explored = HashSet::new();
    let mut count = 0;

    for (name, cur) in graph.nodes.iter() {
        let mut cur_peers: HashSet<_> = cur.peers.difference(&explored).cloned().collect();
        while !cur_peers.is_empty() {
            let peer = cur_peers.iter().next().unwrap().clone();
            let peer_node = graph.nodes.get(&peer).unwrap();
            for shared_peer in peer_node.peers.intersection(&cur_peers) {
                if [name, &peer, shared_peer]
                    .iter()
                    .any(|n| n.starts_with("t"))
                {
                    println!("{}-{}-{}", name, peer, shared_peer);
                    count += 1;
                }
            }
            cur_peers.remove(&peer);
        }
        explored.insert(name.clone());
    }

    println!("Count: {}", count);
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
