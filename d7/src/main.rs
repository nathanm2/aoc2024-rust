use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    let path = env::args().nth(1).unwrap();
    let mut sum: u64 = 0;
    for eq in EquationIter::new(lines(&path)) {
        println!("{}: {:?}", eq.result, eq.values);
        if eq.is_solveable() {
            sum += eq.result;
        }
    }

    println!("Sum: {}", sum);
}

struct Equation {
    result: u64,
    values: Vec<u64>,
}

impl Equation {
    fn is_solveable(&self) -> bool {
        let limit = 3_usize.pow(self.values.len() as u32 - 1);
        for ops in 0..limit {
            if self.try_solution(ops) {
                return true;
            }
        }
        false
    }

    fn try_solution(&self, mut ops: usize) -> bool {
        let mut values_iter = self.values.iter();
        let mut result: u64 = *values_iter.next().unwrap();

        for value in values_iter {
            let op = ops % 3;
            if op == 0 {
                result *= *value;
            } else if op == 1 {
                result += *value;
            } else {
                result = concat(result, *value);
            }
            ops = ops / 3;

            if result > self.result {
                return false;
            }
        }

        result == self.result
    }
}

fn concat(left: u64, right: u64) -> u64 {
    let mut tmp = right / 10;
    let mut digits: u32 = 1;

    while tmp > 0 {
        digits += 1;
        tmp /= 10;
    }

    left * 10u64.pow(digits) + right
}

struct EquationIter<I>
where
    I: Iterator<Item = String>,
{
    iter: I,
}

impl<I> EquationIter<I>
where
    I: Iterator<Item = String>,
{
    fn new(iter: I) -> Self {
        EquationIter { iter }
    }
}

impl<I> Iterator for EquationIter<I>
where
    I: Iterator<Item = String>,
{
    type Item = Equation;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            None => None,
            Some(line) => {
                let mut tokens = line.split_ascii_whitespace();
                let result_str = tokens.next().unwrap();
                let result = result_str[0..result_str.len() - 1].parse::<u64>().unwrap();
                let values: Vec<u64> = tokens.map(|x| x.parse::<u64>().unwrap()).collect();
                Some(Equation { result, values })
            }
        }
    }
}
fn lines(path: &str) -> impl Iterator<Item = String> {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let lines = reader.lines().map(|x| x.unwrap());
    lines
}
