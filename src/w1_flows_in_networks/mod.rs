pub mod p1_evacuating_people;
pub mod p2_crews_to_flights;

extern crate core;

use std::collections::HashMap;
use std::collections::VecDeque;
use std::collections::HashSet;
use std::cmp;

pub struct Edge {
    /// from vertex
    pub from: VertId,
    /// to vertex
    pub to: VertId,
    pub capacity: u16,
}

type VertId = u16;

pub fn max_flow(edges: &[Edge], from: VertId, to: VertId) -> HashMap<VertId, HashMap<VertId, u16>> {
    let mut vert_to_resid_edges = HashMap::new();
    for edge in edges {
        if edge.from != edge.to {
            let dest_to_capacity = vert_to_resid_edges.entry(edge.from).or_insert_with(|| HashMap::new());
            *dest_to_capacity.entry(edge.to).or_insert(0u16) += edge.capacity;
        }
    }

    let mut vert_to_flows: HashMap<VertId, HashMap<VertId, u16>> = HashMap::new();
    loop {
        let path = shortest_path(&vert_to_resid_edges, from, to);

        if path.is_empty() {
            break;
        }

        let min_capacity = min_capacity(&vert_to_resid_edges, &path);

        update_residuals(&mut vert_to_resid_edges, &path, min_capacity);

        update_flows(&mut vert_to_flows, &path, min_capacity);
    }

    vert_to_flows
}

fn shortest_path(vert_to_edges: &HashMap<VertId, HashMap<VertId, u16>>, from: VertId, to: VertId) -> Vec<VertId> {
    let mut tasks = VecDeque::new();
    tasks.push_back(
        BFSTask { vert: from, path_to_vert: Vec::new()}
    );
    let mut visited = HashSet::new();
    visited.insert(from);
    loop {
        let task = tasks.pop_front().unwrap();
        let dests_option = vert_to_edges.get(&task.vert);
        if dests_option.is_some() {
            let dests = dests_option.unwrap().keys();
            for &dest in dests {
                if dest == to {
                    let mut path_to_dest = task.path_to_vert.clone();
                    path_to_dest.push(task.vert);
                    path_to_dest.push(dest);
                    return path_to_dest;
                }
                if !visited.contains(&dest) {
                    visited.insert(dest);
                    // TODO: can (and should?) we reuse prefixes of path?
                    let mut path_to_dest = task.path_to_vert.clone();
                    path_to_dest.push(task.vert);
                    tasks.push_back(
                        BFSTask { vert: dest, path_to_vert: path_to_dest }
                    );
                }
            }
        }

        if tasks.is_empty() {
            break;
        }
    }
    Vec::new()
}

struct BFSTask {
    vert: VertId,
    path_to_vert: Vec<VertId>
}

fn min_capacity(vert_to_edges: &HashMap<VertId, HashMap<VertId, u16>>, path: &[VertId]) -> u16 {
    let mut min_cap = *vert_to_edges.get(&path[0]).unwrap().get(&path[1]).unwrap();
    for i in 2..path.len() {
        min_cap = cmp::min(min_cap, *vert_to_edges.get(&path[i-1]).unwrap().get(&path[i]).unwrap());
    }
    min_cap
}

fn update_residuals(vert_to_resid_edges: &mut HashMap<VertId, HashMap<VertId, u16>>, path: &[VertId], amount: u16) {
    for i in 1..path.len() {
        let from = path[i-1];
        let to = path[i];
        {
            let mut dests = vert_to_resid_edges.get_mut(&from).unwrap();
            let capacity = dests.remove(&to).unwrap();
            if capacity > amount {
                dests.insert(to, capacity - amount);
            }
        }
        let mut dests = vert_to_resid_edges.entry(to).or_insert_with(|| HashMap::new());
        *dests.entry(from).or_insert(0) += amount;
    }
}

fn update_flows(vert_to_flows: &mut HashMap<VertId, HashMap<VertId, u16>>, path: &[VertId], amount: u16) {
    for i in 1..path.len() {
        let from = path[i-1];
        let flows = vert_to_flows.entry(from).or_insert_with(||HashMap::new());
        let to = path[i];
        *flows.entry(to).or_insert(0) += amount;
    }
}
