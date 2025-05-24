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
    let (output, _) = circuit.run(x, y);
    println!("Output: {}", output);

    // Part 2:
    for order in 0..circuit.x_ids.len() {
        let value = 1u64 << order;
        let (o1, _) = circuit.run(value, 0);
        let (o3, _) = circuit.run(value, value);
        if o1 != value {
            println!("{} :: {} + 0 = {}", order, value, o1);
        }
        if o3 != value + value {
            println!("{} :: {} + {} = {}", order, value, value, o3);
        }
    }
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

    fn get<T: CircuitId>(&self, id: &T) -> Option<u8> {
        if id.is_wire() {
            self.wires[id.id()]
        } else {
            self.gates[id.id()]
        }
    }

    fn set<T: CircuitId>(&mut self, id: &T, value: u8) {
        if id.is_wire() {
            self.wires[id.id()] = Some(value);
        } else {
            self.gates[id.id()] = Some(value);
        }
    }

    fn get_wires(&self, ids: &Vec<WireId>) -> Option<u64> {
        let mut value = 0;

        for (order, id) in ids.iter().enumerate() {
            if let Some(b) = self.get(id) {
                value += (b as u64 & 0x1) << order;
            } else {
                return None;
            }
        }

        Some(value)
    }

    fn set_wires(&mut self, ids: &Vec<WireId>, value: u64) {
        for (order, id) in ids.iter().enumerate() {
            self.set(id, ((value >> order) & 0x1) as u8);
        }
    }
}

type GateSet = HashSet<GateId>;
type WireSet = HashSet<WireId>;

impl Circuit {
    fn run(&self, x: u64, y: u64) -> (u64, CircuitState) {
        let mut state = CircuitState::new(self.wires.len(), self.gates.len());
        state.set_wires(&self.x_ids, x);
        state.set_wires(&self.y_ids, y);

        let mut wire_set = WireSet::new();
        wire_set.extend(self.x_ids.iter().map(|&x| x));
        wire_set.extend(self.y_ids.iter().map(|&y| y));

        let mut gate_set = GateSet::new();

        while !wire_set.is_empty() {
            self.update_gate_set(&mut gate_set, &wire_set);
            self.run_gates(&gate_set, &mut state, &mut wire_set);
        }

        (state.get_wires(&self.z_ids).unwrap(), state)
    }

    fn update_gate_set(&self, gate_set: &mut GateSet, wire_set: &WireSet) {
        gate_set.clear();
        gate_set.extend(wire_set.iter().flat_map(|&wire| &self.wires[wire.0].output));
    }

    fn run_gates(&self, gate_set: &GateSet, state: &mut CircuitState, wire_set: &mut WireSet) {
        wire_set.clear();
        for gate_id in gate_set {
            let gate = &self.gates[gate_id.0];
            if let Some(v) = gate.run(state) {
                state.set(&gate.output, v);
                state.set(gate_id, v);
                wire_set.insert(gate.output);
            }
        }
    }
}

trait CircuitId {
    fn is_wire(&self) -> bool;
    fn id(&self) -> usize;
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct WireId(usize);

impl CircuitId for WireId {
    fn is_wire(&self) -> bool {
        true
    }

    fn id(&self) -> usize {
        self.0
    }
}
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct GateId(usize);

impl CircuitId for GateId {
    fn is_wire(&self) -> bool {
        false
    }

    fn id(&self) -> usize {
        self.0
    }
}

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
        let [a, b] = self.input.map(|id| state.get(&id));
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
        if line.starts_with('#') {
            continue;
        } else if line.is_empty() {
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
