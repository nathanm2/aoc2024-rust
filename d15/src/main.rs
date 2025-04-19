use clap::Parser;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::Add;

#[derive(Parser)]
struct Cli {
    /// Input file
    #[arg(short, long, value_name = "INPUT")]
    input: String,

    /// Double wide
    #[arg(short, long)]
    double: bool,
}

fn main() -> Result<(), Box<(dyn Error + 'static)>> {
    let cli = Cli::parse();
    let (mut board, moves) = parse_input_file(&cli.input, cli.double)?;

    board.display();
    for mv in moves {
        board.move_robot(mv);
        // board.display();
    }

    board.display();
    println!("GPS Sum: {}", board.gps_sum());

    Ok(())
}

#[derive(Debug, Copy, Clone)]
enum Move {
    Up,
    Down,
    Left,
    Right,
}

impl From<Move> for Vec2 {
    fn from(value: Move) -> Self {
        match value {
            Move::Up => Vec2 { x: 0, y: -1 },
            Move::Down => Vec2 { x: 0, y: 1 },
            Move::Left => Vec2 { x: -1, y: 0 },
            Move::Right => Vec2 { x: 1, y: 0 },
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Space {
    Empty,
    Box,
    LeftBox,
    RightBox,
    Robot,
    Wall,
}

impl From<Space> for char {
    fn from(value: Space) -> Self {
        match value {
            Space::Empty => '.',
            Space::Box => 'O',
            Space::Robot => '@',
            Space::Wall => '#',
            Space::LeftBox => '[',
            Space::RightBox => ']',
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct Vec2 {
    x: i32,
    y: i32,
}

impl Add<Vec2> for Vec2 {
    type Output = Vec2;

    fn add(self, o: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + o.x,
            y: self.y + o.y,
        }
    }
}

#[derive(Debug)]
struct Board {
    board: Vec<Vec<Space>>,
    width: usize,
    height: usize,
    robot: Vec2,
    dbl: bool,
}

impl Board {
    fn get(&self, pos: Vec2) -> Option<Space> {
        let x = pos.x as usize;
        let y = pos.y as usize;

        (pos.x >= 0 && x < self.width && pos.y >= 0 && y < self.height).then(|| self.board[y][x])
    }

    fn set(&mut self, pos: Vec2, space: Space) {
        let x = pos.x as usize;
        let y = pos.y as usize;

        if pos.x >= 0 && x < self.width && pos.y >= 0 && y < self.height {
            self.board[y][x] = space;
        }
    }

    fn display(&self) {
        let mut s = String::with_capacity((self.width + 1) * self.height);

        for y in 0..self.height {
            for x in 0..self.width {
                s.push(self.board[y][x].into());
            }
            s.push('\n');
        }
        s.push('\n');
        print!("{}", s);
    }

    fn move_robot(&mut self, step: Move) {
        let delta = step.into();
        if self.move_target(self.robot, delta, self.dbl, false) {
            if self.dbl {
                self.move_target(self.robot, delta, false, false);
            }
            self.robot = self.robot + delta;
        }
    }

    fn move_target(&mut self, pos: Vec2, delta: Vec2, check: bool, from_peer: bool) -> bool {
        match self.get(pos).unwrap() {
            Space::Wall => false,
            Space::Empty => true,
            x => {
                let next_pos = pos + delta;
                if self.move_target(next_pos, delta, check, false) {
                    if from_peer == false && delta.y != 0 {
                        let peer_move = if x == Space::LeftBox {
                            self.move_target(pos + Vec2 { x: 1, y: 0 }, delta, check, true)
                        } else if x == Space::RightBox {
                            self.move_target(pos + Vec2 { x: -1, y: 0 }, delta, check, true)
                        } else {
                            true
                        };
                        if peer_move == false {
                            return false;
                        }
                    }
                    if check == false {
                        self.set(next_pos, x);
                        self.set(pos, Space::Empty);
                    }
                    true
                } else {
                    false
                }
            }
        }
    }

    fn gps_sum(&mut self) -> usize {
        let mut sum = 0;
        for y in 0..self.height {
            for x in 0..self.width {
                let b = self.board[y][x];
                if b == Space::Box || b == Space::LeftBox {
                    sum += y * 100 + x
                }
            }
        }
        sum
    }
}

fn parse_input_file(path: &String, dbl: bool) -> Result<(Board, Vec<Move>), Box<dyn Error>> {
    let mut board = Vec::new();
    let mut moves = Vec::new();
    let mut robot = Vec2 { x: 0, y: 0 };

    for (y, line) in BufReader::new(File::open(path)?).lines().enumerate() {
        let line = line?;
        if line.starts_with("#") {
            let row = if dbl {
                parse_board_row_double(&line)?
            } else {
                parse_board_row(&line)?
            };
            if let Some(x) = row.iter().position(|s| *s == Space::Robot) {
                robot = Vec2 {
                    x: x as i32,
                    y: y as i32,
                };
            }
            board.push(row);
        } else {
            moves.append(&mut parse_moves_row(&line)?);
        }
    }

    let width = board[0].len();
    let height = board.len();
    Ok((
        Board {
            board,
            width,
            height,
            robot,
            dbl,
        },
        moves,
    ))
}

fn parse_board_row(line: &str) -> Result<Vec<Space>, String> {
    line.chars()
        .map(|c| match c {
            '#' => Ok(Space::Wall),
            'O' => Ok(Space::Box),
            '.' => Ok(Space::Empty),
            '@' => Ok(Space::Robot),
            _ => Err(format!("Unexpected board character: {}", c)),
        })
        .collect()
}

fn parse_board_row_double(line: &str) -> Result<Vec<Space>, String> {
    let mut row = Vec::new();
    for c in line.chars() {
        match c {
            '#' => {
                row.push(Space::Wall);
                row.push(Space::Wall);
            }

            'O' => {
                row.push(Space::LeftBox);
                row.push(Space::RightBox);
            }

            '.' => {
                row.push(Space::Empty);
                row.push(Space::Empty);
            }

            '@' => {
                row.push(Space::Robot);
                row.push(Space::Empty);
            }

            _ => {
                return Err(format!("Unexpected board character: {}", c));
            }
        }
    }
    Ok(row)
}

fn parse_moves_row(line: &str) -> Result<Vec<Move>, String> {
    line.chars()
        .map(|c| match c {
            '^' => Ok(Move::Up),
            '<' => Ok(Move::Left),
            '>' => Ok(Move::Right),
            'v' => Ok(Move::Down),
            _ => Err(format!("Unexpected direction character: {}", c)),
        })
        .collect()
}
