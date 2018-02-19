pub fn solve_linear_eqs(equations: &[&[f64]]) -> Option<Vec<f64>> {
    if equations.is_empty() {
        return None;
    }

    check_eqs(equations);

    let mut eqs: Vec<Vec<f64>> = equations.iter().map(|row|row.to_vec()).collect();

    for d in 0..eqs.len() {
        if let Some(r) = get_non_zero_eq_idx(&eqs, d) {
            if r != d {
                eqs.swap(r, d);
            }
        } else {
            return None;
        }

        if eqs[d][d] != 1.0 {
            let k = eqs[d][d];
            for c in 0..eqs.len() + 1 {
                eqs[d][c] /= k;
            }
        }

        for r in 0..eqs.len() {
            if r != d {
                let k = eqs[r][d];
                for c in 0..eqs.len() + 1 {
                    eqs[r][c] -= eqs[d][c] * k;
                }
            }
        }
    }

    let mut answers = Vec::with_capacity(equations.len());
    for eq in &eqs {
        answers.push(eq[eq.len() - 1]);
    }
    Some(answers)
}

fn get_non_zero_eq_idx(eqs: &[Vec<f64>], d: usize) -> Option<usize> {
    let mut i = d;
    while eqs[i][d] == 0.0 {
        i += 1;
        if i == eqs.len() {
            return None;
        }
    }
    Some(i)
}

fn check_eqs(equations: &[&[f64]]) {
    for eq in equations {
        if eq.len() != equations.len() + 1 {
            panic!("equation {:?} length ({}) does not equal to number of equations ({}) + 1",
                   eq, eq.len(), equations.len());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use w2_linear_programming::test_utils;
    use w2_linear_programming::mult_vectors;
    use w2_linear_programming::round_f64;

    #[test]
    fn test0() {
        let eqs = Vec::new();

        assert_eq!(solve_linear_eqs(&eqs), None);
    }

    #[test]
    fn test1() {
        let eqs = [
            &[1.0, 0.0, 0.0, 0.0, 1.0][..],
            &[0.0, 1.0, 0.0, 0.0, 5.0][..],
            &[0.0, 0.0, 1.0, 0.0, 4.0][..],
            &[0.0, 0.0, 0.0, 1.0, 3.0][..]
        ];

        assert_eq!(solve_linear_eqs(&eqs).unwrap(), [1.0, 5.0, 4.0, 3.0]);
    }

    #[test]
    fn test2() {
        let eqs = [
            &[1.0, 1.0, 3.0][..],
            &[2.0, 3.0, 7.0][..]
        ];

        assert_eq!(solve_linear_eqs(&eqs).unwrap(), [2.0, 1.0]);
    }

    #[test]
    fn test3() {
        let eqs = [
            &[5.0, -5.0, -1.0][..],
            &[-1.0, -2.0, -1.0][..]
        ];

        let mut answers = solve_linear_eqs(&eqs).unwrap();

        round_answers(&mut answers, 1);
        assert_eq!(answers, [0.2, 0.4]);
    }

    #[test]
    fn test4() {
        let eqs = [
            &[1.0, 1.0, 0.0][..],
            &[1.0, 1.0, 1.0][..]
        ];

        assert_eq!(solve_linear_eqs(&eqs), None);
    }

    fn round_answers(answers: &mut [f64], digits: i32) {
        let tens = 10.0_f64.powi(digits);
        for i in 0..answers.len() {
            answers[i] = (tens * answers[i]).round() / tens;
        }
    }

    #[test]
    fn test_rand() {
        for _ in 0..1000 {
            let eqs: Vec<Vec<f64>> = test_utils::gen_matrix(0, 20, -1000.0, 1000.0);
            // TODO: ugly, maybe solve_linear_eqs must accept &[Vec<f64>]?
            let eqs: Vec<&[f64]> = eqs.iter().map(|row| &(*row)[..]).collect();

            let solution = solve_linear_eqs(&eqs);

            if let Some(solution) = solution {
                for eq in &eqs {
                    let sum = mult_vectors(&solution, &eq[..eq.len() - 1]);
                    let sum_rounded = round_f64(sum, 3);
                    let expected = round_f64(eq[eq.len() - 1], 3);
                    assert_eq!(sum_rounded, expected, "\nequation {:?}\nequations {:?}", eq, eqs);
                }
            }
        }
    }

}