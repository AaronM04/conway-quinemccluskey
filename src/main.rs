use std::fmt;
use std::fmt::Write;

const WIDTH: usize = 6;

#[derive(Clone)]
struct Implicant {
    num_ones:   usize,
    num_dashes: usize,
    ones:       usize,
    dashes:     usize,
    care:       Option<bool>,       // Some(true/false) only used for 0-cubes
    ns:         Vec<usize>,
    used:       bool,
}

// https://doc.rust-lang.org/stable/std/cmp/trait.Eq.html
impl PartialEq for Implicant {
    fn eq(&self, other: &Implicant) -> bool {
        self.ones == other.ones && self.dashes == other.dashes
    }
}

impl Eq for Implicant {}        // I dunno, the docs told me to do this

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
                if !self.used { "*" } else { " " },     // only meaningful at the end
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
            used:       false,
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
        combined_ns.sort();

        Some(Implicant {
            num_ones:   self.num_ones,
            num_dashes: self.num_dashes + 1,
            ones:       self.ones,
            dashes:     self.dashes | ones_xor,
            care:       None,
            ns:         combined_ns,
            used:       false,
        })
    }
}

#[derive(Eq,PartialEq,Debug)]
enum Output {
    Alive,
    Dead,
    DontCare
}


// "full-width" decomposition - 9 bits - top row, middle row (incl. center cell), bottom row
fn decompose(n: usize) -> Output {
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
    if neighbors == 3 || (center && neighbors == 2) {
        Output::Alive
    } else {
        Output::Dead
    }
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
    let mut imps = zerocubes.clone();
    let mut index = 0;
    let mut next_index = imps.len();
    for _ in 1..WIDTH {
        for i in index..(next_index-1) {
            for j in (i+1)..next_index {
                if let Some(imp_new) = imps[i].combine(&imps[j]) {
                    imps[i].used = true;
                    imps[j].used = true;
                    // make sure this isn't a dup
                    let mut dup = false;
                    for k in next_index..imps.len() {
                        if imp_new == imps[k] {
                            dup = true;
                            break;
                        }
                        debug_assert!(imp_new.ones != imps[k].ones || imp_new.dashes != imps[k].dashes);
                    }
                    if !dup {
                        imps.push(imp_new);
                    }
                }
            }
        }
        if next_index == imps.len() {
            break;   // no new Implicants this round
        }
        index = next_index;
        next_index = imps.len();
    }

    // collect the final ones into the output vector, except used ones
    let mut result = Vec::<Implicant>::new();
    for imp in imps.iter() {
        if !imp.used {
            result.push(imp.clone());
        }
    }
    result
}

// From output of part 1 -- this is the sauce ;-)
fn next_gen(n: usize) -> Output {
    let y1 = (n >> 5) & 1;
    let y2 = (n >> 4) & 1;
    let y3 = (n >> 3) & 1;
    let y4 = (n >> 2) & 1;
    let y5 = (n >> 1) & 1;
    let y6 =  n       & 1;
    /* Equivalent to this:
    let term1 = !y1 &  y2 & !y3 & !y4 &  y5 & y6;
    let term2 =  y1 & !y2 & !y3 & !y4       & y6;
    let term3 =  y1 & !y2 & !y3 & !y4 &  y5;
    let term4 =  y1 &  y2 & !y3 & !y4 & !y5;
    let term5 = !y1             &  y4 & !y5 & y6;
    let term6 = !y1 & !y2       &  y4 &  y5;
    let term7 = !y1 &  y2       &  y4 & !y5;
    let slow_result = term1 | term2 | term3 | term4 | term5 | term6 | term7;
    */
    let int1 = !y3 & !y4;
    let result = !y1&y6&(y2&int1&y5 | y4&!y5) | y1&int1&(!y2&(y5 | y6) | y2&!y5) | !y1&y4&(y2^y5);
    //assert!(result == slow_result);
    if result == 1 {
        Output::Alive
    } else {
        Output::Dead
    }
}

// n is 3x3: highest 3 bits (8 thru 6) are row 1, next highest (5 thru 3) are row 2, etc.
//XXX compare against classic(tm) decompose function
fn next_gen_9bit(n: usize) -> Output {
    let a  = (n >> 8) & 1;
    let b  = (n >> 7) & 1;
    let c  = (n >> 6) & 1;
    let d  = (n >> 5) & 1;
    let y6 = (n >> 4) & 1;  // center cell
    let e  = (n >> 3) & 1;
    let f  = (n >> 2) & 1;
    let g  = (n >> 1) & 1;
    let h  =  n       & 1;

    // full adder #1
    let b_xor_c = b^c;
    let y1 = (a & b_xor_c) | (b & c);
    let y2 = a ^ b_xor_c;

    // full adder #2
    let e_xor_f = e^f;
    let c2 = (d & e_xor_f) | (e & f);
    let s2 = d ^ e_xor_f;

    // half adder #1
    let c3 = g & h;
    let s3 = g ^ h;

    // half adder #2
    let c4 = s2 & s3;
    let y5 = s2 ^ s3;

    // full adder #3
    let c2_xor_c3 = c2 ^ c3;
    let y3 = (c4 & c2_xor_c3) | (c2 & c3);
    let y4 = c4 ^ c2_xor_c3;

    next_gen((y1 << 5) |
             (y2 << 4) |
             (y3 << 3) |
             (y4 << 2) |
             (y5 << 1) |
              y6)
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

    println!("\nZero cubes:");
    for imp in zerocubes.iter() {
        println!("{}", imp);
    }

    let prime_imps = calculate_prime_implicants(zerocubes);

    println!("\nPrime Implicants:");
    for imp in prime_imps.iter() {
        println!("{}", imp);
    }

    //////////////////////////////////////////////////////////////////
    // Part 2 : I wrote the following using the output of the above,
    //     after making prime implicant chart (see the .txt file).
    //////////////////////////////////////////////////////////////////

    println!("\n\nPart 2: test the next_gen function against decompose2 (6-bit)");
    let mut failures = 0;
    for n in 0..1<<WIDTH {
        let value = decompose2(n);
        if value == Output::DontCare {
            continue;
        }
        if next_gen(n) != value {
            println!("FAIL n is {:06b} - expected {:?}", n, value);
            failures += 1;
        }
    }
    if failures == 0 {
        println!("PASS");
    }

    //////////////////////////////////////////////////////////////////
    // Part 3 : I wrote next_gen_9bit later, which produces the 6-bit input
    //     for next_gen.
    //////////////////////////////////////////////////////////////////

    println!("\n\nPart 3: test the next_gen_9bit function against decompose (9-bit)");
    let mut failures3 = 0;
    for n in 0..1<<9 {
        let value = decompose(n);
        if value == Output::DontCare { // can't occur with the 9-bit decomposition
            continue;
        }
        if next_gen_9bit(n) != value {
            println!("FAIL n is {:09b} - expected {:?}", n, value);
            failures3 += 1;
        }
    }
    if failures3 == 0 {
        println!("PASS");
    }
}
