fn get_sat(edges: &[[usize; 2]]) -> Vec<Vec<i32>> {
    let num_of_vertices = edges.iter().flat_map(|edge| edge).max().unwrap();

    let var_no = |vertex: usize, freq: usize| -> i32 {
        vertex as i32 + (freq as i32 - 1) * *num_of_vertices as i32
    };

    let mut sat = Vec::with_capacity(num_of_vertices * 4 + edges.len() * 3);
    for vertex in 1..(num_of_vertices + 1) {
        // at least one freq must be assigned
        sat.push(vec!(var_no(vertex, 1), var_no(vertex, 2), var_no(vertex, 3)));

        // at most one freq must be assigned
        sat.push(vec!(-var_no(vertex, 1), -var_no(vertex, 2)));
        sat.push(vec!(-var_no(vertex, 1), -var_no(vertex, 3)));
        sat.push(vec!(-var_no(vertex, 2), -var_no(vertex, 3)));
    }

    for edge in edges {
        // freq must be assigned to one of adjacent vertices
        for freq in 1..4 {
            sat.push(vec!(-var_no(edge[0], freq), -var_no(edge[1], freq)));
        }
    }

    sat
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    use std::fs::File;
    use std::fs::remove_file;
    use std::io::prelude::*;
    use std::process::Command;
    use rand::Rng;
    use rand::thread_rng;
    use std::thread;

    #[test]
    fn test1() {
        let edges = [
            [1, 2],
            [1, 3],
            [2, 3]
        ];

        let sat = get_sat(&edges);

        let sat_solution = solve_sat(&sat);

        assert!(sat_solution.is_some());

        check_solution(&edges, &(sat_solution.unwrap())[..]);
    }

    #[test]
    fn test2() {
        let edges = [
            [1, 2],
            [1, 3],
            [1, 4],
            [2, 3],
            [2, 4],
            [3, 4],
        ];

        let sat = get_sat(&edges);

        let sat_solution = solve_sat(&sat);

        assert!(sat_solution.is_none());
    }

    #[test]
    fn test_rand() {
        for _ in 0..100 {
            let edges = gen_edges();

            let sat = get_sat(&edges);

            let sat_solution = solve_sat(&sat);

            if let Some(sat_solution) = sat_solution {
                check_solution(&edges, &sat_solution);
            }
        }
    }

    fn gen_edges() -> Vec<[usize;2]> {
        let mut rng = thread_rng();

        let vertices_count = rng.gen_range(2, 501);

        let possible_edges_count = (1 + vertices_count) * vertices_count / 2;
        let mut possible_edges = Vec::with_capacity(possible_edges_count);
        for from_vert in 1..vertices_count+1 {
            for to_vert in 1..from_vert {
                possible_edges.push([from_vert, to_vert]);
            }
        }

        let edges_count = rng.gen_range(1, possible_edges.len() / 30 + 2);
        for edge_index in 0..edges_count {
            let rand_edge_index = rng.gen_range(edge_index, possible_edges.len());
            if rand_edge_index != edge_index {
                possible_edges.swap(edge_index, rand_edge_index);
            }
        }
        possible_edges.truncate(edges_count);

        possible_edges
    }

    fn solve_sat(sat: &[Vec<i32>]) -> Option<Vec<i32>> {
        let num_of_vars = sat.iter().flat_map(|row| row).max().unwrap();

        let thread_id = format!("{:?}", thread::current().id());

        let mut in_minisat_path = temp_dir();
        in_minisat_path.push(format!("in_minisat_{}.txt", thread_id));

        {
            let mut in_minisat_file = File::create(&in_minisat_path).expect("Failed to create in minisat file");
            writeln!(in_minisat_file, "p cnf {} {}", num_of_vars, sat.len()).unwrap();
            for vars in sat {
                for var in vars {
                    write!(in_minisat_file, "{} ", var).unwrap();
                }
                writeln!(in_minisat_file, "0").unwrap();
            }
            // TODO: delete for sure
        }

        let mut out_minisat_path = temp_dir();
        out_minisat_path.push(format!("out_minisat_{}.txt", thread_id));

        Command::new("minisat")
            .arg(in_minisat_path.to_str().unwrap())
            .arg(out_minisat_path.to_str().unwrap())
            .output()
            .expect("Failed to run minisat");
        // TODO: delete out file for sure

        remove_file(in_minisat_path).expect("Failed to remove temp in minisat file");

        let mut out_minisat_file = File::open(&out_minisat_path).expect("Failed to open minisat result file");

        let mut contents = String::new();
        out_minisat_file.read_to_string(&mut contents).expect("Failed to read minisat result file");

        remove_file(out_minisat_path).expect("Failed to remove temp minisat result file");

        let lines: Vec<&str> = contents.split('\n').collect();

        if lines[0] == "UNSAT" {
            return None;
        }

        let mut true_vars: Vec<i32> = lines[1].split(' ')
            .map(|var| var.parse().unwrap())
            .collect();
        true_vars.pop(); // minisat ends solution with 0

        Some(true_vars)
    }

    fn check_solution(edges: &[[usize; 2]], sat_solution: &[i32]) {
        let num_of_vertices = sat_solution.len() / 3;
        let mut vert_to_freq = vec![0; num_of_vertices + 1];
        for vert in 1..num_of_vertices + 1 {
            for freq in 1..4 {
                let solution_index = vert + (freq - 1) * num_of_vertices - 1;
                if sat_solution[solution_index] > 0 {
                    assert_eq!(0, vert_to_freq[vert], "Vertex {} was already assigned frequency {}", vert, vert_to_freq[vert]);
                    vert_to_freq[vert] = freq;
                }
            }
        }

        for edge in edges {
            assert_ne!(vert_to_freq[edge[0]], vert_to_freq[edge[1]],
                       "Vertices {} and {} share same frequency {}", edge[0], edge[1], vert_to_freq[0]);
        }
    }
}