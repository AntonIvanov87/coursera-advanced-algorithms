fn get_sat(edges: &[[usize; 2]]) -> Vec<Vec<i32>> {
    let num_of_vertices: usize = edges.iter().map(|edge| edge[0].max(edge[1])).max().unwrap();

    let var_no = |vertex: usize, freq: usize| -> i32 {
        (vertex + (freq - 1) * num_of_vertices) as i32
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
    use w3_np_completeness::test_utils::solve_sat;
    use w3_np_completeness::test_utils::gen_edges;

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
            let edges = gen_edges(500, | max_edges | max_edges / 30 + 2);

            let sat = get_sat(&edges);

            let sat_solution = solve_sat(&sat);

            if let Some(sat_solution) = sat_solution {
                check_solution(&edges, &sat_solution);
            } else {
                // TODO: check there is indeed no solution
            }
        }
    }



    fn check_solution(edges: &[[usize; 2]], sat_solution: &[usize]) {
        let vertices_count = edges.iter().map(|edge| edge[0].max(edge[1])).max().unwrap();

        let mut vert_to_freq = vec![0; vertices_count + 1];
        for var_no in sat_solution {
            let freq = (var_no-1) / vertices_count + 1;
            let vert = var_no - (freq-1) * vertices_count;
            assert_eq!(vert_to_freq[vert], 0,
                       "Can not assign frequency {} to vertex {} because frequency {} was already assigned to it", freq, vert, vert_to_freq[vert]);
            vert_to_freq[vert] = freq;
        }

        for vert in 1..vertices_count {
            assert_ne!(vert_to_freq[vert], 0,
                       "No frequency was assigned to vertex {}", vert);
        }

        for edge in edges {
            assert_ne!(vert_to_freq[edge[0]], vert_to_freq[edge[1]],
                       "Vertices {} and {} share same frequency {}", edge[0], edge[1], vert_to_freq[0]);
        }
    }
}