use std::env;
use std::fs;

fn main() {
    let path = env::args().nth(1).unwrap();
    let message = fs::read_to_string(path).unwrap();
    let mut parser = Parser::new();
    let mut sm_do = StrMatch::new("do()");
    let mut sm_dont = StrMatch::new("don't()");
    let mut enable = true;
    let mut sum = 0;
    for ch in message.chars() {
        if true == sm_do.push_char(ch) {
            println!("do()");
            enable = true;
        }
        if true == sm_dont.push_char(ch) {
            println!("don't()");
            enable = false;
        }
        if let Some((a, b)) = parser.push_char(ch) {
            if enable {
                println!("mul({},{})", a, b);
                sum += a * b;
            }
        }
    }
    println!("Sum: {}", sum);
}

struct Parser {
    next: char,
    num: Option<u32>,
    save: Option<u32>,
}

impl Parser {
    fn new() -> Self {
        Parser {
            next: 'm',
            num: None,
            save: None,
        }
    }

    fn push_char(&mut self, c: char) -> Option<(u32, u32)> {
        let mut result = None;
        self.next = if c == 'm' {
            self.num = None;
            self.save = None;
            'u'
        } else if c == self.next {
            match c {
                'u' => 'l',
                'l' => '(',
                '(' => ',',
                ',' if self.num.is_some() => {
                    self.save = self.num;
                    self.num = None;
                    ')'
                }
                ')' if self.num.is_some() => {
                    result = Some((self.save.unwrap(), self.num.unwrap()));
                    'm'
                }
                _ => 'm',
            }
        } else if c.is_ascii_digit() && (self.next == ',' || self.next == ')') {
            let value = c.to_digit(10).unwrap();
            self.num = Some(self.num.unwrap_or(0) * 10 + value);
            self.next
        } else {
            'm'
        };
        result
    }
}

struct StrMatch {
    v: Vec<char>,
    i: usize,
}

impl StrMatch {
    fn new(s: &str) -> Self {
        StrMatch {
            v: s.chars().collect(),
            i: 0,
        }
    }

    fn push_char(&mut self, c: char) -> bool {
        if c == self.v[self.i] {
            self.i += 1;
            if self.i == self.v.len() {
                self.i = 0;
                return true;
            }
        } else {
            self.i = 0;
        }
        false
    }
}
