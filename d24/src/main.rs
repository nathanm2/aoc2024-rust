#![allow(dead_code)]

use clap::Parser;
use regex::Regex;
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
    let (circuit, mut state) = parse_file(cli.file)?;
    run(&circuit, &mut state);

    Ok(())
}

fn run(circuit: &Circuit, state: &mut WireState) {
    let mut changed: HashSet<WireId> = state.keys().map(|&x| x).collect();
    while !changed.is_empty() {}
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct WireId(usize);

#[derive(Copy, Clone)]
struct GateId(usize);

enum GateOp {
    And,
    Or,
    Xor,
}
struct Gate {
    op: GateOp,
    input: [WireId; 2],
    out: WireId,
}

struct Wire {
    name: String,
    out: Vec<GateId>,
}

impl Wire {
    fn new(name: &str) -> Self {
        Wire {
            name: name.to_owned(),
            out: Vec::new(),
        }
    }
}

struct Circuit {
    wires: Vec<Wire>,
    gates: Vec<Gate>,
}

type WireState = HashMap<WireId, bool>;

struct CircuitBuilder {
    wire_names: HashMap<String, WireId>,
    wires: Vec<Wire>,
    gates: Vec<Gate>,
    wire_state: WireState,
}

impl CircuitBuilder {
    fn new() -> Self {
        CircuitBuilder {
            wire_names: HashMap::new(),
            wires: Vec::new(),
            gates: Vec::new(),
            wire_state: HashMap::new(),
        }
    }

    fn get_wire_id(&mut self, name: &str) -> WireId {
        if let Some(&id) = self.wire_names.get(name) {
            id
        } else {
            let id = WireId(self.wires.len());
            self.wires.push(Wire::new(name));
            self.wire_names.insert(name.to_owned(), id);
            id
        }
    }

    fn add_wire(&mut self, name: &str, state: &str) -> Result<(), String> {
        let state = match state {
            "0" => false,
            "1" => true,
            _ => return Err(format!("Invalid wire value: {} : {}", name, state)),
        };
        let id = self.get_wire_id(name);
        self.wire_state.insert(id, state);
        Ok(())
    }

    fn add_gate(&mut self, left: &str, op: &str, right: &str, out: &str) -> Result<(), String> {
        let left = self.get_wire_id(left);
        let right = self.get_wire_id(right);
        let out = self.get_wire_id(out);
        let op = match op {
            "AND" => GateOp::And,
            "OR" => GateOp::Or,
            "XOR" => GateOp::Xor,
            _ => return Err(format!("Invalid component")),
        };

        let gate_id = self.gates.len();
        let gate = Gate {
            op,
            input: [left, right],
            out,
        };
        self.gates.push(gate);
        self.wires[left.0].out.push(GateId(gate_id));
        self.wires[right.0].out.push(GateId(gate_id));

        Ok(())
    }

    fn build(self) -> (Circuit, WireState) {
        (
            Circuit {
                wires: self.wires,
                gates: self.gates,
            },
            self.wire_state,
        )
    }
}

fn parse_file(path: String) -> Result<(Circuit, WireState), Box<dyn Error>> {
    let wire_re = Regex::new(r"(.*): (\d)")?;
    let comp_re = Regex::new(r"(\w+) (\w+) (\w+) -> (\w+)")?;
    let mut builder = CircuitBuilder::new();

    let mut wire_mode = true;
    for line in BufReader::new(File::open(path)?).lines() {
        let line = line?;
        if line.is_empty() {
            wire_mode = false;
        } else if wire_mode {
            let cap = wire_re.captures(&line).ok_or_else(|| "Invalid input")?;
            builder.add_wire(&cap[1], &cap[2])?;
        } else {
            let cap = comp_re.captures(&line).ok_or_else(|| "Invalid input")?;
            builder.add_gate(&cap[1], &cap[2], &cap[3], &cap[4])?;
        }
    }
    Ok(builder.build())
}
