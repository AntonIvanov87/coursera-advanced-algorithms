use std::f64;
use std::iter;
use w2_linear_programming::Solution;
use w2_linear_programming::round_around_0;

pub fn solve_simplex(a_orig: &[&[f64]], b_orig: &[f64], c_orig: &[f64]) -> Solution {
    // TODO: degenerate case
    // TODO: use internally something with higher precision than f64

    let init_result = init_simplex(a_orig, b_orig, c_orig);
    match init_result {
        None => Solution::None,
        Some((mut aa_sl, mut bb_sl, mut cc_sl, mut row_to_basic_col)) => {
            println!();
            println!("found initial slack form:");
            println!("aa {:?}", aa_sl);
            println!("row to basic col {:?}", row_to_basic_col);
            println!("bb {:?}", bb_sl);
            println!("cc {:?}", cc_sl);

            let slack_solution = rewrite_slack(&mut aa_sl, &mut bb_sl, &mut cc_sl, &mut row_to_basic_col);

            if let Solution::Some(xx_sl) = slack_solution {
                // TODO: do not copy, just pop and shrink
                let xx = (&xx_sl[..c_orig.len()]).to_vec();
                Solution::Some(xx)
            } else {
                slack_solution
            }
        }
    }

}

// TODO: return struct instead of tuple
fn init_simplex(aa_orig: &[&[f64]], bb_orig: &[f64], cc_orig: &[f64]) -> Option<(Vec<Vec<f64>>, Vec<f64>, Vec<f64>, Vec<usize>)> {

    // sl = slack form
    let mut aa_sl: Vec<Vec<f64>> = aa_orig.iter()
        .map(|r_orig| {
            let mut r = r_orig.to_vec();
            // TODO: do not extend: basic var is always 1 in its' row and zero in others.
            // TODO: but do not break x_aux_col
            r.extend(iter::repeat(0.0).take(aa_orig.len()));
            r
        })
        .collect();
    for row in 0..aa_sl.len() {
        let col = aa_orig[row].len() + row;
        aa_sl[row][col] = 1.0;
    }

    let mut bb_sl = bb_orig.to_vec();

    let mut cc_sl = cc_orig.to_vec();
    cc_sl.extend(iter::repeat(0.0).take(bb_orig.len()));

    let mut row_to_basic_col: Vec<usize> = (cc_orig.len()..cc_orig.len() + aa_orig.len()).collect();

    let mut b_i_min = 0;
    let mut b_min = bb_sl[b_i_min];
    for i in 1..bb_sl.len() {
        if bb_sl[i] < b_min {
            b_min = bb_sl[i];
            b_i_min = i;
        }
    }
    // If b_min is non negative - current slack form is ok
    if b_min >= 0.0 {
        return Some((aa_sl, bb_sl, cc_sl, row_to_basic_col));
    }
    // Rewrite current slack form in order basic solution to be feasible

    let x_aux_col = aa_sl[0].len();
    for row in 0..aa_sl.len() {
        aa_sl[row].push(-1.0);
    }

    let mut cc_aux = vec![0.0; cc_sl.len()];
    cc_aux.push(-1.0);

    pivot(&mut aa_sl, &mut bb_sl, &mut cc_aux, &mut row_to_basic_col, b_i_min, x_aux_col);

    let slack_solution = rewrite_slack(&mut aa_sl, &mut bb_sl, &mut cc_aux, &mut row_to_basic_col);

    if let Solution::Some(xx_aux) = slack_solution {
        // TODO: round
        if xx_aux[x_aux_col] != 0.0 {
            return None;
        }

        let x_aux_row_opt = row_to_basic_col.iter().position(|col| *col == x_aux_col);
        if let Some(leaving_row) = x_aux_row_opt {
            let entering_col = if leaving_row == 0 { 1 } else { 0 };
            pivot(&mut aa_sl, &mut bb_sl, &mut cc_aux, &mut row_to_basic_col, leaving_row, entering_col);
        }

        // we do not need x_aux anymore
        for row in 0..aa_sl.len() {
            aa_sl[row].pop();
        }

        // remove basic vars from objective function
        for row in 0..row_to_basic_col.len() {
            let col = row_to_basic_col[row];
            // TODO: round
            if cc_sl[col] != 0.0 {
                let k = cc_sl[col];
                for col2 in 0..cc_sl.len() {
                    if col2 == col {
                        cc_sl[col2] = 0.0;
                    } else {
                        cc_sl[col2] = k.mul_add(-aa_sl[row][col2], cc_sl[col2]);
                    }
                }
            }
        }

        Some((aa_sl, bb_sl, cc_sl, row_to_basic_col))
    } else if let Solution::None = slack_solution {
        None
    } else {
        panic!("Unbounded aux form");

    }
}

fn rewrite_slack(aa: &mut [Vec<f64>], bb: &mut [f64], cc: &mut [f64], row_to_basic_col: &mut [usize]) -> Solution {
    loop {

        println!();
        println!("rewriting slack form");
        println!("aa {:?}", aa);
        println!("row_to_basic_col {:?}", row_to_basic_col);
        println!("bb {:?}", bb);
        println!("cc {:?}", cc);

        if let Some(entering_col) = col_with_positive_k_in_cc(&cc) {
            println!("entering col {}", entering_col);
            if let Some(leaving_row) = bounding_row(&bb, &aa, entering_col) {
                println!("leaving row {}", leaving_row);
                // TODO: is it possible, can we eliminate it earlier?
                if bb[leaving_row] < 0.0 {
                    return Solution::None;
                } else {
                    pivot(aa, bb, cc, row_to_basic_col, leaving_row, entering_col);
                }
            } else {
                return Solution::Unbounded;
            }
        } else {
            break;
        }
    }

    let mut xx: Vec<f64> = vec![0.0; cc.len()];
    for row in 0..row_to_basic_col.len() {
        if bb[row].is_sign_negative() {
            return Solution::None;
        }
        let col = row_to_basic_col[row];
        xx[col] = bb[row];
    }
    Solution::Some(xx)
}

fn col_with_positive_k_in_cc(cc: &[f64]) -> Option<usize> {
    for col in 0..cc.len() {
        if cc[col] > 0.0 {
            return Some(col);
        }
    }
    None
}

fn bounding_row(bb: &[f64], aa: &[Vec<f64>], col: usize) -> Option<usize> {
    let mut row: usize = 0;
    while row < aa.len() {
        if aa[row][col] > 0.0 {
            break;
        }
        row += 1;
    }
    if row == bb.len() {
        return None;
    }
    let mut min_row = row;
    let mut min_b: f64 = bb[row] / aa[row][col];
    for row in min_row + 1..aa.len() {
        if aa[row][col] > 0.0 {
            let new_b = bb[row] / aa[row][col];
            if new_b < min_b {
                min_row = row;
                min_b = new_b;
            }
        }
    }

    Some(min_row)
}

fn pivot(aa: &mut [Vec<f64>], bb: &mut [f64], cc: &mut [f64], row_to_basic_col: &mut [usize], leaving_row: usize, entering_col: usize) {
    row_to_basic_col[leaving_row] = entering_col;

    let k = aa[leaving_row][entering_col];
    for col in 0..aa[leaving_row].len() {
        if col == entering_col {
            aa[leaving_row][col] = 1.0;
        } else {
            aa[leaving_row][col] /= k;
        }
    }
    bb[leaving_row] = bb[leaving_row] / k;

    for row in 0..aa.len() {
        if row != leaving_row {
            let k: f64 = aa[row][entering_col];
            for col in 0..aa[row].len() {
                if col == entering_col {
                    aa[row][col] = 0.0;
                } else {
                    aa[row][col] = k.mul_add(-aa[leaving_row][col], aa[row][col]);
                }
            }
            bb[row] = k.mul_add(-bb[leaving_row], bb[row]);
        }
    }

    let k: f64 = cc[entering_col];
    for col in 0..cc.len() {
        // TODO: still need this if?
        if col == entering_col {
            cc[col] = 0.0;
        } else {
            cc[col] = round_around_0(k.mul_add(-aa[leaving_row][col], cc[col]));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use w2_linear_programming::mult_vectors;
    use w2_linear_programming::round_f64;
    use w2_linear_programming::test_utils;


    #[test]
    fn test1() {
        let a = [
            &[-1.0, -1.0][..],
            &[1.0, 0.0][..],
            &[0.0, 1.0][..]
        ];
        let b = [-1.0, 2.0, 2.0];
        let c = [-1.0, 2.0];

        let sol = solve_simplex(&a, &b, &c);

        assert_eq!(sol, Solution::Some(vec![0.0, 2.0]));
    }

    #[test]
    fn test2() {
        let a = [
            &[1.0, 1.0][..],
            &[-1.0, -1.0][..],
        ];
        let b = [-1.0, -2.0];
        let c = [1.0, 1.0];

        let sol = solve_simplex(&a, &b, &c);

        assert_eq!(sol, Solution::None);
    }

    #[test]
    fn test3() {
        let a = [
            &[0.0, 0.0, 1.0][..]
        ];
        let b = [3.0];
        let c = [1.0, 1.0, 1.0];

        let sol = solve_simplex(&a, &b, &c);

        assert_eq!(sol, Solution::Unbounded);
    }

    #[test]
    fn test4() {
        let a = [
            &[-6.0][..]
        ];
        let b = [-3.0];
        let c = [-4.0];

        let sol = solve_simplex(&a, &b, &c);

        assert_eq!(sol, Solution::Some(vec![0.5]));
    }

    #[test]
    fn test5() {
        let a = [
            &[-0.16][..]
        ];
        let b = [-2.14];
        let c = [2.56];

        let sol = solve_simplex(&a, &b, &c);

        assert_eq!(sol, Solution::Unbounded);
    }

    #[test]
    fn test_rand() {
        for _ in 0..1000 {
            let (a, b, c) = test_utils::gen_linear_prog(100, -100.0, 100.0, -1_000_000.0, 1_000_000.0, -100.0, 100.0);
            let a: Vec<&[f64]> = a.iter().map(|r| &(*r)[..]).collect();

            let solution = solve_simplex(&a, &b, &c);

            // no solution, infinity and optimality is checked in p2_optimal_diet
            // here we limit to just check feasibility
            if let Solution::Some(xx) = solution {
                if xx[0].is_infinite() {
                    continue;
                }

                for row in 0..a.len() {
                    let sum = mult_vectors(&xx, a[row]);
                    assert!(round_f64(sum, 3) <= round_f64(b[row], 3),
                            "infeasible solution {:?}\ninequality {:?} <= {:?}\n{:?} <= {:?}\na {:?}\nb {:?}\nc {:?}",
                            xx, a[row], b[row], sum, b[row], a, b, c);
                }
            }
        }
    }
}