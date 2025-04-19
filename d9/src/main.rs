use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::env;
use std::fs::File;
use std::io::Read;

fn main() {
    let path = env::args().nth(1).unwrap();
    let (mut files, mut slots) = parse_disk_map(&path);
    defrag(&mut files, &mut slots);
    println!("Checksum: {}", files.checksum());
}

struct FreeSlots {
    bins: [BinaryHeap<Reverse<usize>>; 9],
}

impl FreeSlots {
    fn new() -> Self {
        FreeSlots {
            bins: std::array::from_fn(|_| BinaryHeap::new()),
        }
    }

    fn push(&mut self, position: usize, span: usize) {
        (span > 0).then(|| self.bins[span - 1].push(Reverse(position)));
    }

    fn find_free(&self, span: usize) -> Option<(usize, usize)> {
        self.bins[span - 1..]
            .iter()
            .enumerate()
            .filter_map(|(i, bin)| bin.peek().map(|pos| (i + span - 1, pos.0)))
            .min_by_key(|&(_, pos)| pos)
            .map(|(index, pos)| (index, pos))
    }

    fn claim_free(&mut self, bin: usize, span: usize) {
        let pos = self.bins[bin].pop().unwrap().0;
        self.push(pos + span, bin + 1 - span);
    }
}

#[derive(Debug)]
struct DiskFile {
    id: usize,
    pos: usize,
    span: usize,
}

impl DiskFile {
    fn checksum(&self) -> usize {
        let result = (0..self.span).map(|i| (i + self.pos) * self.id).sum();
        result
    }
}

struct FileVec {
    id: usize,
    files: Vec<DiskFile>,
}

impl FileVec {
    fn new() -> Self {
        FileVec {
            id: 0,
            files: Vec::<DiskFile>::new(),
        }
    }

    fn push(&mut self, pos: usize, span: usize) {
        self.files.push(DiskFile {
            id: self.id,
            pos,
            span,
        });
        self.id += 1;
    }

    fn checksum(&self) -> usize {
        self.files.iter().map(|f| f.checksum()).sum()
    }
}

fn defrag(filevec: &mut FileVec, slots: &mut FreeSlots) {
    for f in filevec.files.iter_mut().rev() {
        if let Some((bin, pos)) = slots.find_free(f.span) {
            if pos < f.pos {
                slots.claim_free(bin, f.span);
                f.pos = pos;
            }
        }
    }
}

fn parse_disk_map(path: &String) -> (FileVec, FreeSlots) {
    let mut files = FileVec::new();
    let mut slots = FreeSlots::new();
    let mut pos = 0;

    for (index, span) in disk_map_values(&path).enumerate() {
        let span = span as usize;
        if index % 2 == 0 {
            files.push(pos, span)
        } else {
            slots.push(pos, span)
        }
        pos += span;
    }
    (files, slots)
}

fn disk_map_values(path: &str) -> impl Iterator<Item = u32> {
    let file = File::open(path).unwrap();
    file.bytes()
        .map(|b| b.unwrap() as char)
        .filter_map(|c| c.to_digit(10))
}
