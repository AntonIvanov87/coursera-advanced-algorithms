use w1_flows_in_networks::VertId;
use w1_flows_in_networks::max_flow;
use w1_flows_in_networks::Edge;

pub fn max_flow_amount(edges: &[Edge], from: VertId, to: VertId) -> u32 {
    let vert_to_flows = max_flow(edges, from, to);
    match vert_to_flows.get(&from) {
        Some(flows) => flows.values().map(|a| *a as u32).sum(),
        None => 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let graph = vec![
            Edge { from: 1, to: 2, capacity: 2 },
            Edge { from: 2, to: 5, capacity: 5 },
            Edge { from: 1, to: 3, capacity: 6 },
            Edge { from: 3, to: 4, capacity: 2 },
            Edge { from: 4, to: 5, capacity: 1 },
            Edge { from: 3, to: 2, capacity: 3 },
            Edge { from: 2, to: 4, capacity: 1 },
        ];

        let max_flow_amount = max_flow_amount(&graph, 1, 5);

        assert_eq!(max_flow_amount, 6);
    }

    #[test]
    fn test2() {
        let graph = vec![
            Edge { from: 1, to: 2, capacity: 10_000 },
            Edge { from: 1, to: 3, capacity: 10_000 },
            Edge { from: 2, to: 3, capacity: 1 },
            Edge { from: 3, to: 4, capacity: 10_000 },
            Edge { from: 2, to: 4, capacity: 10_000 },
        ];

        let max_flow_amount = max_flow_amount(&graph, 1, 4);

        assert_eq!(max_flow_amount, 20_000);
    }
}