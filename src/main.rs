use std::fmt::Write;

#[derive(Debug)]
struct TermInfo {
    num_ones: usize,
    binary:   String,
    care:     bool,
    minterm:  String,
    n:        usize,
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

// narrower decomposition - 6 bits:
//          543210
//          yy      - sum of top row
//            yyy   - sum of W, E, bottom row (values 6 and 7 are don't care)
//               y  - center cell
// Output: Some(true/false) -> the output matters; None -> don't care
fn decompose2(n: usize) -> Option<bool> {
    let neighbors1 = (n >> 4) & 3;
    let neighbors2 = (n >> 1) & 7;
    let center: bool = (n & 1) == 1;
    let neighbors = neighbors1 + neighbors2;
    if neighbors2 <= 5 {
        Some(neighbors == 3 || (center && neighbors == 2))
    } else {
        // don't care
        None
    }
}

fn info(n: usize, width: usize, care: bool) -> TermInfo {
    let mut num_ones = 0;
    let mut binary = String::with_capacity(width);
    let mut shift: isize = (width as isize) - 1;
    while shift >= 0 {       // while shift >= 0
        if ((n >> shift) & 1) == 1 {
            binary.push('1');
            num_ones += 1;
        } else {
            binary.push('0');
        }
        shift -= 1;
    }
    let mut minterm = String::new();
    write!(&mut minterm, "m({})", n).unwrap();
    TermInfo {
        num_ones: num_ones,
        binary:   binary,
        care:     care,
        minterm:  minterm,
        n:        n
    }
}

fn main() {
    /*
    for n in 0..512 {
        let value = decompose(n);
        if value {
            println!("{:?}", info(n, 9, true));
        }
    }
    */
    let mut terminfos: Vec<TermInfo> = Vec::new();
    const WIDTH: usize = 6;
    for n in 0..(1<<WIDTH) {
        let value = decompose2(n);
        if value != Some(false) {
            let ti = match value {
                Some(true)  => info(n, WIDTH, true),
                None        => info(n, WIDTH, false),
                Some(false) => panic!("can't happen")
            };
            //println!("{:?}", ti);
            terminfos.push(ti);
        }
    }

    println!("Size 2 implicants");
    //XXX Keep a set of all ns that were combined, and then loop through terminfos at
    //    end to save the "no further" ones.
    let mut s2implicants: Vec<(String, Vec<usize>)> = Vec::new();
    for i in 0..(terminfos.len()-1) {
        let ti_i = &terminfos[i];
        for j in (i+1)..(terminfos.len()) {
            let ti_j = &terminfos[j];
            let mut s = String::with_capacity(WIDTH);
            let mut ns: Vec<usize> = Vec::new();
            let mut hamming_dist = 0;
            let mut differs_by_dash = false;      // actually, I don't think this can happen
            for k in 0..WIDTH {
                let i_char = ti_i.binary.chars().nth(k).unwrap();
                let j_char = ti_j.binary.chars().nth(k).unwrap();
                if i_char == j_char {
                    s.push(i_char);
                } else {
                    if i_char == '-' {
                        differs_by_dash = true;
                    }
                    hamming_dist += 1;
                    s.push('-');
                }
            }
            if hamming_dist == 1 && !differs_by_dash {
                ns.push(ti_i.n);
                ns.push(ti_j.n);
                println!("{:?}", (s.clone(), ns.clone()));
                s2implicants.push((s, ns));
            }
        }
    }

    println!("Size 4 implicants");
    //XXX
}
