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

    /// X argument.
    #[arg(short)]
    x: Option<u64>,

    /// Y argument.
    #[arg(short)]
    y: Option<u64>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let (circuit, x, y) = parse_file(cli.file)?;
    let x = cli.x.unwrap_or(x);
    let y = cli.y.unwrap_or(y);

    // Part 1:
    let (output, _state) = circuit.run(x, y);
    println!("Output: {}", output);

    Ok(())
}

struct Circuit {
    wires: Vec<Wire>,
    gates: Vec<Gate>,
    x_ids: Vec<WireId>,
    y_ids: Vec<WireId>,
    z_ids: Vec<WireId>,
}

struct CircuitState {
    wires: Vec<Option<u8>>,
    gates: Vec<Option<u8>>,
}

impl CircuitState {
    fn new(wire_sz: usize, gate_sz: usize) -> Self {
        CircuitState {
            wires: vec![None; wire_sz],
            gates: vec![None; gate_sz],
        }
    }

    fn get_wire(&self, id: WireId) -> Option<u8> {
        self.wires[id.0]
    }
}

type GateSet = HashSet<GateId>;
type WireSet = HashSet<WireId>;

impl Circuit {
    fn run(&self, x: u64, y: u64) -> (u64, CircuitState) {
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
    output: WireId,
}

impl Gate {
    fn run(&self, state: &CircuitState) -> Option<u8> {
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
    input: Option<GateId>,
    output: Vec<GateId>,
}

impl Wire {
    fn new(name: &str) -> Self {
        Wire {
            name: name.to_owned(),
            input: None,
            output: Vec::new(),
        }
    }
}

struct CircuitBuilder {
    wire_names: HashMap<String, WireId>,
    wires: Vec<Wire>,
    gates: Vec<Gate>,
    x: u64,
    y: u64,
}

impl CircuitBuilder {
    fn new() -> Self {
        CircuitBuilder {
            wire_names: HashMap::new(),
            wires: Vec::new(),
            gates: Vec::new(),
            x: 0,
            y: 0,
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

    fn add_wire(&mut self, name: &str, state: &str) -> Result<(), Box<dyn Error>> {
        let state = match state {
            "0" => 0,
            "1" => 1,
            _ => return Err(format!("Invalid wire value: {} : {}", name, state))?,
        };

        let order = name[1..].parse::<u32>()?;
        let value = state * 2u64.pow(order);
        if name.starts_with('x') {
            self.x += value;
        } else {
            self.y += value;
        }

        let _ = self.get_wire_id(name);
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
            output: out,
        });
        self.wires[left.0].output.push(gate_id);
        self.wires[right.0].output.push(gate_id);
        self.wires[out.0].input = Some(gate_id);
        Ok(())
    }

    fn wire_ids(&self, ch: char) -> Vec<WireId> {
        let mut ids: Vec<_> = self
            .wires
            .iter()
            .enumerate()
            .filter(|(_, wire)| wire.name.starts_with(ch))
            .collect();
        ids.sort_by_key(|(_, wire)| &wire.name);
        ids.iter().map(|(id, _)| WireId(*id)).collect()
    }

    fn build(self) -> (Circuit, u64, u64) {
        let x_ids = self.wire_ids('x');
        let y_ids = self.wire_ids('y');
        let z_ids = self.wire_ids('z');
        (
            Circuit {
                wires: self.wires,
                gates: self.gates,
                x_ids,
                y_ids,
                z_ids,
            },
            self.x,
            self.y,
        )
    }
}

fn parse_file(path: String) -> Result<(Circuit, u64, u64), Box<dyn Error>> {
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
