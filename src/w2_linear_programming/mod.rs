mod p1_energy_values;
mod p2_optimal_diet;
mod p3_online_ads;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum Solution {
    None,
    Unbounded,
    Some(Vec<f64>)
}

fn mult_vectors(a: &[f64], b: &[f64]) -> f64 {
    let mut s = 0.0;
    for i in 0..a.len() {
        s = a[i].mul_add(b[i], s);
    }
    s
}

pub fn round_around_0(f: f64) -> f64 {
    if round_f64(f, 10).abs() == 0.0 { 0.0 } else { f }
}

pub fn round_f64(answer: f64, digits: i32) -> f64 {
    let tens = 10.0_f64.powi(digits);
    (tens * answer).round() / tens
}

pub fn round_slice_f64(slice: &[f64], digits: i32) -> Vec<f64> {
    slice.iter().map(|x|round_f64(*x, digits)).collect()
}

#[cfg(test)]
pub mod test_utils {

    use super::*;
    use rand;
    use rand::Rng;

    pub fn gen_matrix(min_rows: usize, max_rows: usize, min_a: f64, max_a: f64) -> Vec<Vec<f64>> {
        let mut rng = rand::thread_rng();
        let eqs_count = rng.gen_range(min_rows, max_rows+1);
        let mut eqs: Vec<Vec<f64>> = Vec::with_capacity(eqs_count);
        for _ in 0..eqs_count {
            let eq = gen_vec(eqs_count+1, min_a, max_a);
            eqs.push(eq);
        }
        eqs
    }

    pub fn gen_linear_prog(max_rows: usize, min_a: f64, max_a: f64, min_b: f64, max_b: f64, min_c: f64, max_c: f64)
                       -> (Vec<Vec<f64>>, Vec<f64>, Vec<f64>) {

        let mut aa: Vec<Vec<f64>> = gen_matrix(1, max_rows, min_a, max_a);
        // TODO: ugly and not efficient
        for row in &mut aa {
            row.pop();
        }
        let bb: Vec<f64> = gen_vec(aa.len(), min_b, max_b);
        let mut cc: Vec<f64>;
        loop {
            cc = gen_vec(aa[0].len(), min_c, max_c);
            if cc.iter().any(|c| c.abs() != 0.0) {
                break;
            }
        }
        (aa, bb, cc)
    }

    pub fn gen_vec(size: usize, min_val: f64, max_val: f64) -> Vec<f64> {
        let mut rng = rand::thread_rng();
        let mut v: Vec<f64> = Vec::with_capacity(size);
        for _ in 0..size {
            v.push(round_around_0(round_f64(rng.gen_range(min_val, max_val), 1)));
        }
        v
    }

}