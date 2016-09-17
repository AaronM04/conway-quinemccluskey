use std::fmt::Write;

#[derive(Debug)]
struct TermInfo {
    num_ones: usize,
    minterm: String,
    binary: String
}

fn decompose(n: usize) -> bool {
    let mut neighbors = 0;
    let mut center = false;
    for row in 0..3 {
        for col in 0..3 {
            let value: usize = ((n >> (3*(2-row))) >> (2-col)) & 1;
            if col == 1 && row == 1 {
                center = value==1;
            } else if value == 1 {
                neighbors += 1;
            }
        }
    }
    neighbors == 3 || (center && neighbors == 2)
}

fn info(n: usize) -> TermInfo {
    let mut num_ones = 0;
    let mut binary = String::new();
    binary.reserve(9);
    let mut shift = 8;
    while shift >= 0 {
        if ((n >> shift) & 1) == 1 {
            binary.push('1');
            num_ones += 1;
        } else {
            binary.push('0');
        }
        shift -= 1;
    }
    let mut minterm = String::new();
    write!(&mut minterm, "m{:03}", n).unwrap();
    TermInfo { num_ones: num_ones, minterm: minterm, binary: binary }
}

fn main() {
    for n in 0..512 {
        let value = decompose(n);
        if value {
            println!("{:?}", info(n));
        }
    }
}
