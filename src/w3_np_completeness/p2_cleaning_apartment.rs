fn get_sat(edges: &[[usize; 2]]) -> Vec<Vec<i32>> {
    let vertices_count: usize = edges.iter().map(|edge| edge[0].max(edge[1])).max().unwrap();
    let moves_count = vertices_count;

    let var_no = |vertex: usize, move_no: usize| -> i32 {
        (vertex + (move_no - 1) * vertices_count) as i32
    };

    let cleaned_at_least_once_clauses = vertices_count;
    let cleaned_at_most_once_clauses = vertices_count * moves_count * (moves_count - 1) / 2;
    let not_simultaneous_clauses = moves_count * vertices_count * (vertices_count - 1) / 2;
    let corridor_clauses = vertices_count * (moves_count - 1);
    let mut sat: Vec<Vec<i32>> = Vec::with_capacity(cleaned_at_least_once_clauses + cleaned_at_most_once_clauses + not_simultaneous_clauses + corridor_clauses);

    for vert in 1..vertices_count + 1 {
        let cleaned_at_least_once_clause: Vec<i32> = (1..moves_count + 1)
            .map(|move_no| var_no(vert, move_no) as i32)
            .collect();
        sat.push(cleaned_at_least_once_clause);

        // cleaned at most once
        for move_no1 in 1..moves_count {
            for move_no2 in move_no1 + 1..moves_count + 1 {
                sat.push(vec!(-var_no(vert, move_no1), -var_no(vert, move_no2)));
            }
        }
    }

    // not simultaneous
    for move_no in 1..moves_count + 1 {
        for vert1 in 1..vertices_count {
            for vert2 in vert1 + 1..vertices_count + 1 {
                sat.push(vec!(-var_no(vert1, move_no), -var_no(vert2, move_no)));
            }
        }
    }

    let directed_edges = get_directed_edges(edges, vertices_count);

    for from_vert in 1..vertices_count + 1 {
        for move_no in 1..moves_count {
            let mut at_most_one_edge_clause = Vec::with_capacity(directed_edges[from_vert].len() + 1);
            at_most_one_edge_clause.push(-var_no(from_vert, move_no));
            for to_vert in directed_edges[from_vert].iter() {
                at_most_one_edge_clause.push(var_no(*to_vert, move_no + 1));
            }
            sat.push(at_most_one_edge_clause);
        }
    }

    sat
}

fn get_directed_edges(edges: &[[usize; 2]], vertices_count: usize) -> Vec<Vec<usize>> {
    // keep fake vertex = 0 just for convenience
    let mut directed_edges: Vec<Vec<usize>> = Vec::with_capacity(vertices_count + 1);
    for _ in 0..vertices_count + 1 {
        directed_edges.push(Vec::new());
    }
    for edge in edges {
        directed_edges[edge[0]].push(edge[1]);
        directed_edges[edge[1]].push(edge[0]);
    }
    directed_edges
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::iter::FromIterator;
    use w3_np_completeness::test_utils::solve_sat;
    use w3_np_completeness::test_utils::gen_edges;

    #[test]
    fn test1() {
        let edges = [
            [1, 2],
            [2, 3],
            [3, 5],
            [4, 5]
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
        ];

        let sat = get_sat(&edges);

        let sat_solution = solve_sat(&sat);

        assert!(sat_solution.is_none(), "True vars {:?}", sat_solution.unwrap());
    }

    #[test]
    fn test_rand() {
        for _ in 0..100 {
            // TODO: check UNSAT is not often
            let edges = gen_edges(30, |max_edges| max_edges);

            let sat = get_sat(&edges);

            let sat_solution = solve_sat(&sat);

            if let Some(sat_solution) = sat_solution {
                check_solution(&edges, &sat_solution);
            }
        }
    }

    fn check_solution(edges: &[[usize; 2]], sat_solution: &[usize]) {
        let vertices_count: usize = edges.iter().map(|edge| edge[0].max(edge[1])).max().unwrap();
        let moves_count = vertices_count;

        let mut move_to_vert = vec![0; moves_count + 1];  // keep move no = 0 for convenience
        for var_no in sat_solution {
            let move_no = (var_no-1) / vertices_count + 1;
            let vert = var_no - (move_no-1) * vertices_count;
            assert_eq!(move_to_vert[move_no], 0,
                       "Vertex {} can not be cleaned at move {} because vertex {} was cleaned at that move", vert, move_no, move_to_vert[move_no]);
            move_to_vert[move_no] = vert;
        }

        let directed_edges: Vec<HashSet<usize>> = get_directed_edges(edges, vertices_count).iter()
            .map(|to_verts| HashSet::from_iter(to_verts.iter().cloned()))
            .collect();

        assert!(move_to_vert[1] > 0, "No room is cleaned at move {}", 1);
        for move_no in 2..moves_count + 1 {
            assert!(move_to_vert[move_no] > 0, "No room is cleaned at move {}", move_no);
            let prev_vert = move_to_vert[move_no - 1];
            let cur_vert = move_to_vert[move_no];
            assert!(directed_edges[prev_vert].contains(&cur_vert),
                    "There is no edge from vertex {} to vertex {}", prev_vert, cur_vert);
        }
    }
}
