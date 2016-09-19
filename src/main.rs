use std::fmt;
use std::fmt::Write;

const WIDTH: usize = 6;

struct Implicant {
    num_ones:   usize,
    num_dashes: usize,
    ones:       usize,
    dashes:     usize,
    care:       Option<bool>,       // Some(true/false) only used for 0-cubes
    ns:         Vec<usize>,
}

impl fmt::Display for Implicant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        debug_assert!(self.num_ones   <= WIDTH);
        debug_assert!(self.num_dashes <= WIDTH);
        debug_assert!(self.num_ones + self.num_dashes <= WIDTH);
        debug_assert!(self.ones   < (1 << WIDTH));
        debug_assert!(self.dashes < (1 << WIDTH));
        let mut s = String::with_capacity(WIDTH);
        for shift in (0..WIDTH).rev() {
            let bit = (self.ones >> shift) & 1;
            let dashbit = (self.dashes >> shift) & 1;
            if dashbit == 1 {
                debug_assert!(bit == 0);
                s.push('-');
            } else if bit == 1 {
                s.push('1');
            } else {
                s.push('0');
            }
        }
        let mut mt_expr = String::new();
        if self.ns.len() == 1 {
            debug_assert!(self.ones == self.ns[0]);
            write!(&mut mt_expr, "m{}", self.ns[0]).unwrap();
        } else {
            mt_expr.push_str("m(");
            for ref n in &self.ns {
                let mut s_tmp = String::new();
                write!(&mut s_tmp, "{},", n).unwrap();
                mt_expr.push_str(&s_tmp);
            }
            mt_expr.push_str(")");
        }
        write!(f, "Implicant( 1s:{} -s:{} {} {}{})",
                self.num_ones,
                self.num_dashes,
                s,
                mt_expr,
                if self.care == Some(false) { " DONTCARE" } else { "" }
            )
    }
}

#[inline]
fn count_ones(n: usize) -> usize {
    debug_assert!(n < (1 << WIDTH));
    let mut tmp = n;
    let mut num_ones = 0;
    for _ in 0..WIDTH {
        if (tmp & 1) == 1 {
            num_ones += 1;
        }
        tmp >>= 1;
    }
    num_ones
}

impl Implicant {
    fn new0cube(n: usize, care: bool) -> Implicant {
        Implicant{
            num_ones:   count_ones(n),
            num_dashes: 0,
            ones:       n,
            dashes:     0,
            care:       Some(care),
            ns:         vec![n]
        }
    }

    fn combine(&self, other: &Implicant) -> Option<Implicant> {
        let onesdiff = self.num_ones as isize - other.num_ones as isize;
        if onesdiff.abs() != 1 || self.dashes != other.dashes {
            return None;
        }
        if self.num_ones > other.num_ones {
            return other.combine(self);
        }
        let ones_xor   = (self.ones & !self.dashes) ^ (other.ones & !other.dashes);
        if count_ones(ones_xor) != 1 {
            return None;
        }

        let mut combined_ns = self.ns.clone();
        for ref other_n in &other.ns {      // There's got to be another way to do this
            combined_ns.push(**other_n);
        }

        Some(Implicant {
            num_ones:   self.num_ones,
            num_dashes: self.num_dashes + 1,
            ones:       self.ones,
            dashes:     self.dashes | ones_xor,
            care:       None,
            ns:         combined_ns
        })
    }
}

#[derive(Eq,PartialEq)]
enum Output {
    Alive,
    Dead,
    DontCare
}

// narrower decomposition - 6 bits:
//          543210
//          yy      - sum of top row
//            yyy   - sum of W, E, bottom row (values 6 and 7 are don't care)
//               y  - center cell
// Output: Alive / Dead / DontCare
fn decompose2(n: usize) -> Output {
    let neighbors1 = (n >> 4) & 3;
    let neighbors2 = (n >> 1) & 7;
    let center: bool = (n & 1) == 1;
    let neighbors = neighbors1 + neighbors2;
    if neighbors2 <= 5 {
        if neighbors == 3 || (center && neighbors == 2) {
            Output::Alive
        } else {
            Output::Dead
        }
    } else {
        Output::DontCare
    }
}

fn main() {
    let mut zerocubes: Vec<Implicant> = Vec::new();
    for n in 0..(1<<WIDTH) {
        let value = decompose2(n);
        if value == Output::Alive || value == Output::DontCare {
            let imp = Implicant::new0cube(n, value != Output::DontCare);
            zerocubes.push(imp);
        }
    }

    println!("Size 2 implicants");
    //XXX Keep a parallel array of count of all zerocubes that were combined, and
    // then loop through zerocubes at end to save the "no further" ones.
    let mut s2implicants: Vec<Implicant> = Vec::new();
    for i in 0..(zerocubes.len()-1) {
        let imp_i = &zerocubes[i];
        for j in (i+1)..(zerocubes.len()) {
            let imp_j = &zerocubes[j];
            if let Some(imp_new) = imp_i.combine(imp_j) {
                //XXX increment comb count for imp_i and imp_j in parallel array
                println!("{}", imp_new);
                s2implicants.push(imp_new);
            }
        }
    }

    println!("Size 4 implicants");
    //XXX
}




// "full-width" decomposition - 9 bits - top row, middle row (incl. center cell), bottom row
/*
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
*/
