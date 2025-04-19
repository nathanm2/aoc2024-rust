#![allow(dead_code)]

use clap::Parser;
use log::debug;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Parser)]
struct Cli {
    /// Input file
    #[arg(short, long, value_name = "INPUT")]
    input: String,
}

fn main() -> Result<(), Box<(dyn Error + 'static)>> {
    env_logger::init();
    let cli = Cli::parse();
    let (regs, mem) = parse_input_file(&cli.input)?;

    let output = run_program(regs, &mem)?;
    println!("Output: {}", output);

    println!("Find Match: {}", find_match(&mem)?);
    Ok(())
}

fn run_program(regs: [i64; 3], mem: &Vec<u8>) -> Result<String, Box<dyn Error>> {
    let mut computer = Computer {
        reg: regs,
        pc: 0,
        mem,
        out: Vec::new(),
    };

    computer.run()?;
    let output_str = computer
        .out
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(",");

    Ok(output_str)
}

fn find_match(mem: &Vec<u8>) -> Result<i64, Box<dyn Error>> {
    if let Some(result) = find_next_match(mem, mem.len() - 1, 0)? {
        Ok(result)
    } else {
        Err("No match found")?
    }
}

fn find_next_match(
    mem: &Vec<u8>,
    memidx: usize,
    start: i64,
) -> Result<Option<i64>, Box<dyn Error>> {
    for i in 0..8 {
        let mut computer = Computer {
            reg: [start + i, 0, 0],
            pc: 0,
            mem,
            out: Vec::new(),
        };

        if let Some(x) = computer.run_to_out()? {
            if x == mem[memidx] {
                if memidx == 0 {
                    return Ok(Some(start + i));
                } else {
                    if let Some(result) = find_next_match(mem, memidx - 1, (start + i) << 3)? {
                        return Ok(Some(result));
                    }
                }
            }
        }
    }

    Ok(None)
}

#[derive(Copy, Clone, Debug)]
enum Reg {
    A = 0,
    B = 1,
    C = 2,
}

#[derive(Debug)]
struct Computer<'a> {
    reg: [i64; 3],
    pc: usize,
    mem: &'a Vec<u8>,
    out: Vec<i64>,
}

impl Computer<'_> {
    fn run(&mut self) -> Result<(), String> {
        while self.step()? == true {}
        Ok(())
    }

    fn run_to_out(&mut self) -> Result<Option<u8>, String> {
        let cur = self.out.len();
        while self.step()? == true {
            if cur + 1 == self.out.len() {
                return Ok(Some(self.out[cur] as u8));
            }
        }
        Ok(None)
    }

    fn step(&mut self) -> Result<bool, String> {
        if let Some((opcode, operand)) = self.fetch() {
            self.execute(opcode.try_into()?, operand)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn read_reg(&self, reg: Reg) -> i64 {
        self.reg[reg as usize]
    }

    fn write_reg(&mut self, reg: Reg, value: i64) {
        self.reg[reg as usize] = value;
    }

    fn combo_operand(&self, operand: u8) -> Result<i64, String> {
        match operand {
            0 => Ok(0),
            1 => Ok(1),
            2 => Ok(2),
            3 => Ok(3),
            4 => Ok(self.read_reg(Reg::A)),
            5 => Ok(self.read_reg(Reg::B)),
            6 => Ok(self.read_reg(Reg::C)),
            _ => Err(format!("Invalid operand: {}", operand)),
        }
    }

    fn fetch(&mut self) -> Option<(u8, u8)> {
        if let Some(opcode) = self.mem.get(self.pc) {
            if let Some(operand) = self.mem.get(self.pc + 1) {
                self.pc += 2;
                return Some((*opcode, *operand));
            }
        }
        None
    }

    fn div(&mut self, reg: Reg, opcode: OpCode, operand: u8) -> Result<(), String> {
        let numerator = self.read_reg(Reg::A);
        let denominator = self.combo_operand(operand)?;
        let result = numerator >> denominator;
        self.write_reg(reg, numerator >> denominator);
        debug!(
            "{:?} {}: Reg{:?}={:08x} RegA[{:08x}] >> Combo[{:08x}]",
            opcode, operand, reg, result, numerator, denominator
        );
        Ok(())
    }

    fn execute(&mut self, opcode: OpCode, operand: u8) -> Result<(), String> {
        match opcode {
            OpCode::Adv => {
                self.div(Reg::A, opcode, operand)?;
            }
            OpCode::Bxl => {
                let b = self.read_reg(Reg::B);
                let o = operand as i64;
                let r = b ^ o;
                self.write_reg(Reg::B, r);
                debug!(
                    "{:?} {}: RegB={:08x} RegB[{:08x}] ^ {:08x}",
                    opcode, operand, r, b, o
                );
            }
            OpCode::Bst => {
                let c = self.combo_operand(operand)?;
                let r = c % 8;
                self.write_reg(Reg::B, r);
                debug!(
                    "{:?} {}: RegB={:08x} Combo[{:08x}] % 8",
                    opcode, operand, r, c
                );
            }
            OpCode::Jnz => {
                let a = self.read_reg(Reg::A);

                if a != 0 {
                    self.pc = operand as usize;
                }
                debug!(
                    "{:?} {}: PC={:08x} RegA[{:08x}]",
                    opcode, operand, self.pc, a
                );
            }
            OpCode::Bxc => {
                let b = self.read_reg(Reg::B);
                let c = self.read_reg(Reg::C);
                let r = b ^ c;
                self.write_reg(Reg::B, b ^ c);
                debug!(
                    "{:?} {}: RegB={:08x} RegB[{:08x}] ^ RegC[{:08x}]",
                    opcode, operand, r, b, c
                );
            }
            OpCode::Out => {
                let c = self.combo_operand(operand)?;
                let r = c % 8;
                self.out.push(r);
                debug!(
                    "{:?} {}: OUT={:08x} Combo[{:08x}] % 8",
                    opcode, operand, r, c
                );
            }
            OpCode::Bdv => {
                self.div(Reg::B, opcode, operand)?;
            }
            OpCode::Cdv => {
                self.div(Reg::C, opcode, operand)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
enum OpCode {
    Adv,
    Bxl,
    Bst,
    Jnz,
    Bxc,
    Out,
    Bdv,
    Cdv,
}

impl TryFrom<u8> for OpCode {
    type Error = String;

    fn try_from(i: u8) -> Result<Self, Self::Error> {
        match i {
            0 => Ok(OpCode::Adv),
            1 => Ok(OpCode::Bxl),
            2 => Ok(OpCode::Bst),
            3 => Ok(OpCode::Jnz),
            4 => Ok(OpCode::Bxc),
            5 => Ok(OpCode::Out),
            6 => Ok(OpCode::Bdv),
            7 => Ok(OpCode::Cdv),
            _ => Err(format!("Invalid opcode: {}", i)),
        }
    }
}

fn parse_register_line(line: &str) -> Result<i64, Box<dyn Error>> {
    line.split(' ')
        .nth(2)
        .ok_or_else(|| format!("Invalid register line: {}", line))?
        .parse::<i64>()
        .map_err(|e| e.into())
}

fn parse_memory_line(line: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    line.split(' ')
        .nth(1)
        .ok_or_else(|| format!("Invalid program line: {}", line))?
        .split(',')
        .map(|v| v.parse::<u8>().map_err(|e| e.into()))
        .collect()
}

fn parse_input_file(path: &String) -> Result<([i64; 3], Vec<u8>), Box<dyn Error>> {
    let mut reg = [0; 3];
    let mut lines = BufReader::new(File::open(path)?).lines();

    // Parse registers
    for i in 0..3 {
        reg[i] = parse_register_line(&lines.next().ok_or("Missing register line")??)?;
    }

    // Skip empty line
    lines.next().ok_or("Missing empty line")??;

    // Parse memory
    let mem = parse_memory_line(&lines.next().ok_or("Missing memory line")??)?;

    Ok((reg, mem))
}
