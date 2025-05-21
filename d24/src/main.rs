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

    // Part 1:
    circuit.run(&mut state);
    let output = circuit.output(&state);
    println!("Part 1: {}", output);

    Ok(())
}

struct Circuit {
    wires: Vec<Wire>,
    gates: Vec<Gate>,
}

type GateSet = HashSet<GateId>;
type WireSet = HashSet<WireId>;

impl Circuit {
    fn run(&self, state: &mut WireState) {
        let mut wire_set = self.initial_wire_set(state);
        let mut gate_set = GateSet::new();

        while !wire_set.is_empty() {
            self.update_gate_set(&mut gate_set, &wire_set);
            let mut changes = HashSet::new();
            for gate_id in &gate_set {
                let gate = &self.gates[gate_id.0];
                if let Some(v) = gate.run(state) {
                    state[gate.out.0] = Some(v);
                    changes.insert(gate.out);
                }
            }
            wire_set = changes;
        }
    }

    fn output(&self, wire_state: &WireState) -> u64 {
        let mut z_wires: Vec<_> = self
            .wires
            .iter()
            .enumerate()
            .filter(|(_, wire)| wire.name.starts_with("z"))
            .collect();
        z_wires.sort_by_key(|(_, wire)| &wire.name);

        z_wires
            .into_iter()
            .map(|(idx, _)| wire_state[idx])
            .enumerate()
            .filter_map(|(order, value)| value.map(|v| v as u64 * 2u64.pow(order as u32)))
            .sum()
    }

    fn initial_wire_set(&self, state: &WireState) -> WireSet {
        state
            .iter()
            .enumerate()
            .take_while(|(_, s)| s.is_some())
            .map(|(idx, _)| WireId(idx))
            .collect()
    }

    fn update_gate_set(&self, gate_set: &mut GateSet, wire_set: &WireSet) {
        gate_set.clear();
        gate_set.extend(wire_set.iter().flat_map(|&wire| &self.wires[wire.0].out));
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct WireId(usize);

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
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

impl Gate {
    fn run(&self, wire_state: &WireState) -> Option<u8> {
        let [a, b] = self.input.map(|id| wire_state[id.0]);
        match (a, b) {
            (Some(a), Some(b)) => Some(match self.op {
                GateOp::And => a & b,
                GateOp::Or => a | b,
                GateOp::Xor => a ^ b,
            }),
            _ => None,
        }
    }
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

type WireState = Vec<Option<u8>>;

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
            wire_state: Vec::new(),
        }
    }

    fn get_wire_id(&mut self, name: &str) -> WireId {
        if let Some(&id) = self.wire_names.get(name) {
            id
        } else {
            let id = WireId(self.wires.len());
            self.wires.push(Wire::new(name));
            self.wire_state.push(None);
            self.wire_names.insert(name.to_owned(), id);
            id
        }
    }

    fn add_wire(&mut self, name: &str, state: &str) -> Result<(), String> {
        let state = match state {
            "0" => 0,
            "1" => 1,
            _ => return Err(format!("Invalid wire value: {} : {}", name, state)),
        };
        let wire_id = self.get_wire_id(name);
        self.wire_state[wire_id.0] = Some(state);
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
            _ => return Err("Invalid component".into()),
        };

        let gate_id = GateId(self.gates.len());
        self.gates.push(Gate {
            op,
            input: [left, right],
            out,
        });
        self.wires[left.0].out.push(gate_id);
        self.wires[right.0].out.push(gate_id);
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
            let cap = wire_re.captures(&line).ok_or("Invalid input")?;
            builder.add_wire(&cap[1], &cap[2])?;
        } else {
            let cap = comp_re.captures(&line).ok_or("Invalid input")?;
            builder.add_gate(&cap[1], &cap[2], &cap[3], &cap[4])?;
        }
    }
    Ok(builder.build())
}
