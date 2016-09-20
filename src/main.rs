use std::fmt;
use std::fmt::Write;
use std::collections::HashMap;

const WIDTH: usize = 6;

#[derive(Clone)]
struct Implicant {
    num_ones:   usize,
    num_dashes: usize,
    ones:       usize,
    dashes:     usize,
    care:       Option<bool>,       // Some(true/false) only used for 0-cubes
    ns:         Vec<usize>,
    is_final:   bool,
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
            for &n in self.ns.iter() {
                let mut s_tmp = String::new();
                write!(&mut s_tmp, "{},", n).unwrap();
                mt_expr.push_str(&s_tmp);
            }
            mt_expr.push_str(")");
        }
        write!(f, "Implicant( 1s:{} -s:{} {}{} {}{} )",
                self.num_ones,
                self.num_dashes,
                s,
                if self.is_final { "*" } else { " " },
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
            ns:         vec![n],
            is_final:   false,
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
        for &other_n in other.ns.iter() {
            combined_ns.push(other_n);
        }

        Some(Implicant {
            num_ones:   self.num_ones,
            num_dashes: self.num_dashes + 1,
            ones:       self.ones,
            dashes:     self.dashes | ones_xor,
            care:       None,
            ns:         combined_ns,
            is_final:   false,
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

fn calculate_prime_implicants(zerocubes: Vec<Implicant>) -> Vec<Implicant> {
    let mut imp_hash = HashMap::new();      // maps size to vector of Implicants
    imp_hash.insert(1, zerocubes.clone());
    for shift in 1..WIDTH {
        let prev_size     = 1 << (shift-1);
        let mut prev_imps = imp_hash.get_mut(&prev_size).unwrap();
        if prev_imps.len() == 0 {
            break;
        }

        let size          = 1 << shift;
        imp_hash.insert(size, Vec::<Implicant>::new());
        let mut imps      = imp_hash.get_mut(&size).unwrap();

        let mut used_prev_imps = vec![0usize; prev_imps.len()];  // parallel to prev_imps, # times used
        let mut imps: Vec<Implicant> = Vec::new();
        for i in 0..(prev_imps.len()-1) {
            let imp_i = &prev_imps[i];
            for j in (i+1)..(prev_imps.len()) {
                let imp_j = &prev_imps[j];
                if let Some(imp_new) = imp_i.combine(imp_j) {
                    used_prev_imps[i] += 1;
                    used_prev_imps[j] += 1;
                    imps.push(imp_new);
                }
            }
        }
        for i in 0..prev_imps.len() {
            if used_prev_imps[i] == 0 {
                prev_imps[i].is_final = true;
            }
        }


    }

    // mark largest size as all final
    for shift in (0..WIDTH).rev() {
        if let Some(imps) = imp_hash.get(&(1<<shift)) {
            for ref mut imp in imps.iter() {
                imp.is_final = true;
            }
            break;
        }
    }

    // 2) collect the final ones into the output vector
    let mut result: Vec<Implicant> = Vec::new();
    for shift in 0..WIDTH {

    }
    // XXX
    result
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

    let prime_imps = calculate_prime_implicants(zerocubes);

    for imp in prime_imps.iter() {
        println!("{}", imp);
    }
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
