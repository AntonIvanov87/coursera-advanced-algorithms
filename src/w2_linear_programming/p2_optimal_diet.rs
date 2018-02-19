use std::f64;
use w2_linear_programming::mult_vectors;
use w2_linear_programming::p1_energy_values::solve_linear_eqs;
use w2_linear_programming::round_f64;
use w2_linear_programming::round_slice_f64;
use w2_linear_programming::Solution;

const UNBOUNDED_B_VAL: f64 = 10e9;

fn solve_brute_force(a_orig: &[&[f64]], b_orig: &[f64], c_orig: &[f64]) -> Solution {
    let mut ab: Vec<Vec<f64>> = Vec::with_capacity(a_orig.len());
    for r in 0..a_orig.len() {
        let mut row = a_orig[r].to_vec();
        row.push(b_orig[r]);
        ab.push(row);
    }
    for c in 0..c_orig.len() {
        let mut row = vec![0.0; c_orig.len() + 1];
        row[c] = -1.0;
        ab.push(row);
    }
    {
        let mut row = vec![1.0; c_orig.len()];
        row.push(UNBOUNDED_B_VAL);
        ab.push(row);
    }
    let ab: Vec<&[f64]> = ab.iter().map(|r| &(*r)[..]).collect();

    if let Some(vertex_sol) = pick_rows_and_solve(&[], &ab, &[], c_orig) {
        if vertex_sol.has_unbounded_eq {
            Solution::Unbounded
        } else {
            Solution::Some(vertex_sol.x)
        }
    } else {
        Solution::None
    }
}

fn pick_rows_and_solve(equations: &[&[f64]], inequalities: &[&[f64]], discarded: &[&[f64]], c: &[f64])
                       -> Option<VertexSolution> {
    if equations.len() == c.len() {
        if let Some(solution) = solve_linear_eqs(equations) {
            for ineq in inequalities.iter().chain(discarded.iter()) {
                if !check_inequality(ineq, &solution) {
                    return None;
                }
            }

            let has_unbounded_eq = equations.iter().any(|eq| eq[eq.len() - 1] == UNBOUNDED_B_VAL);

            let objective_val: f64 = mult_vectors(&solution, c);

//            println!("found solution {:?} with target val {}", solution, objective_val);

            return Some(VertexSolution { x: solution, objective_val, has_unbounded_eq });
        } else {
            return None;
        }
    }

    if c.len() - equations.len() == inequalities.len() {
        let mut new_use_rows = equations.to_vec();
        new_use_rows.extend_from_slice(inequalities);
        return pick_rows_and_solve(&new_use_rows, &[][..], discarded, c);
    }

    let less_inequalities: &[&[f64]] = &inequalities[1..];

    let mut new_use_rows = equations.to_vec();
    new_use_rows.push(inequalities[0]);
    let sol_with = pick_rows_and_solve(&new_use_rows, less_inequalities, discarded, c);

    let mut new_discarded = discarded.to_vec();
    new_discarded.push(inequalities[0]);
    let sol_without = pick_rows_and_solve(equations, less_inequalities, &new_discarded, c);

    match (sol_with, sol_without) {
        (None, None) => None,
        (Some(s), None) => Some(s),
        (None, Some(s)) => Some(s),
        (Some(s_w), Some(s_wo)) => {
            if s_w.objective_val >= s_wo.objective_val {
                Some(s_w)
            } else {
                Some(s_wo)
            }
        }
    }
}

struct VertexSolution {
    x: Vec<f64>,
    objective_val: f64,
    has_unbounded_eq: bool,
}

fn check_inequality(ineq: &[f64], solution: &[f64]) -> bool {
    let s = mult_vectors(&solution, &ineq[..ineq.len() - 1]);
    // TODO: why 10?
    return round_f64(s, 10) <= round_f64(ineq[ineq.len() - 1], 10);
}

#[cfg(test)]
mod tests {
    use super::*;
    use w2_linear_programming::p3_online_ads::solve_simplex;
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

        let sol = solve_brute_force(&a, &b, &c);

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

        let sol = solve_brute_force(&a, &b, &c);

        assert_eq!(sol, Solution::None);
    }

    #[test]
    fn test3() {
        let a = [
            &[0.0, 0.0, 1.0][..]
        ];
        let b = [3.0];
        let c = [1.0, 1.0, 1.0];

        let sol = solve_brute_force(&a, &b, &c);

        assert_eq!(sol, Solution::Unbounded);
    }

    #[test]
    fn test4() {
        let a = [
            &[-6.0][..]
        ];
        let b = [-3.0];
        let c = [-4.0];

        let sol = solve_brute_force(&a, &b, &c);
        assert_eq!(sol, Solution::Some(vec![0.5]));
    }

    #[test]
    fn test5() {
        let a = [
            &[-0.16][..]
        ];
        let b = [-2.14];
        let c = [2.56];

        let sol = solve_brute_force(&a, &b, &c);

        assert_eq!(sol, Solution::Unbounded);
    }

    #[test]
    fn test_rand() {
        for _ in 0..1000 {
            println!("\n------------------------------");
            // TODO: remove
//            let (a, b, c) = test_utils::gen_linear_prog(8, -100.0, 100.0, -1_000_000.0, 1_000_000.0, -100.0, 100.0);
            let (a, b, c) = test_utils::gen_linear_prog(2, -10.0, 10.0, -10.0, 10.0, -10.0, 10.0);
            let a: Vec<&[f64]> = a.iter().map(|r| &(*r)[..]).collect();

            let brute_force_solution = solve_brute_force(&a, &b, &c);

            let simplex_solution = solve_simplex(&a, &b, &c);

            if let (&Solution::Some(ref bf_xx), &Solution::Some(ref sx_xx)) = (&brute_force_solution, &simplex_solution) {
                let bf_xx_rounded = round_slice_f64(&bf_xx, 10);
                let sx_xx_rounded = round_slice_f64(&sx_xx, 10);
                assert_eq!(bf_xx_rounded, sx_xx_rounded, "\naa {:?}\nbb {:?}\ncc{:?}", a, b, c);

            } else {
                assert_eq!(brute_force_solution, simplex_solution, "\naa {:?}\nbb {:?}\ncc{:?}", a, b, c);
            }

        }
    }
}