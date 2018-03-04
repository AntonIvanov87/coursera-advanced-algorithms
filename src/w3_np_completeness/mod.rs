mod p1_assign_frequencies;
mod p2_cleaning_apartment;

#[cfg(test)]
mod test_utils {
    use rand::Rng;
    use rand::thread_rng;
    use std::env::temp_dir;
    use std::fs::File;
    use std::fs::remove_file;
    use std::io::prelude::*;
    use std::process::Command;
    use std::thread;

    pub fn gen_edges(max_vertices: usize, max_edges: fn(usize) -> usize) -> Vec<[usize; 2]> {
        let mut rng = thread_rng();

        let vertices_count = rng.gen_range(2, max_vertices+1);

        let possible_edges_count = vertices_count * (vertices_count - 1) / 2;
        let mut possible_edges = Vec::with_capacity(possible_edges_count);
        for from_vert in 1..vertices_count {
            for to_vert in from_vert+1..vertices_count+1 {
                possible_edges.push([from_vert, to_vert]);
            }
        }

        let edges_count = rng.gen_range(1, max_edges(possible_edges_count) + 1);
        for edge_index in 0..edges_count {
            let rand_edge_index = rng.gen_range(edge_index, possible_edges.len());
            if rand_edge_index != edge_index {
                possible_edges.swap(edge_index, rand_edge_index);
            }
        }
        possible_edges.truncate(edges_count);

        possible_edges
    }

    pub fn solve_sat(sat: &[Vec<i32>]) -> Option<Vec<usize>> {
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

        let true_vars: Vec<usize> = lines[1].split(' ')
            .map(|var_str| var_str.parse::<i32>().unwrap())
            .filter(|var| *var > 0)
            .map(|var| var as usize)
            .collect();

        Some(true_vars)
    }
}