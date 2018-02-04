use std::collections::HashMap;
use w1_flows_in_networks::max_flow;
use w1_flows_in_networks::Edge;
use w1_flows_in_networks::VertId;
use std::collections::HashSet;

pub fn crew_to_flight(crew_to_possible_flights: HashMap<u16, Vec<u16>>) -> HashMap<u16, u16> {
    let source_vert_id = 0u16;

    let max_crew = crew_to_possible_flights.keys()
        .map(|k| *k)
        .max().unwrap();

    let max_flight: u16 = crew_to_possible_flights.values()
        .map(|flights| if flights.is_empty() {
            0u16
        } else {
            *flights.iter().max().unwrap()
        })
        .max().unwrap();

    let flight_as_vert_id = |flight: u16| max_crew + flight;
    let vert_id_as_flight = |vert_id: VertId| vert_id - max_crew;

    let target_vert_id = flight_as_vert_id(max_flight) + 1;

    let mut edges = Vec::new();
    let mut flights_connected_to_target = HashSet::new();
    for (&crew, possible_flights) in crew_to_possible_flights.iter() {
        if !possible_flights.is_empty() {
            edges.push(Edge { from: source_vert_id, to: crew, capacity: 1 });
            for &possible_flight in possible_flights {
                edges.push(Edge { from: crew, to: flight_as_vert_id(possible_flight), capacity: 1 });
                if !flights_connected_to_target.contains(&possible_flight) {
                    edges.push(Edge { from: flight_as_vert_id(possible_flight), to: target_vert_id, capacity: 1 });
                    flights_connected_to_target.insert(possible_flight);
                }
            }
        }
    }

    let vert_id_to_flows = max_flow(&edges, source_vert_id, target_vert_id);

    let mut crew_to_flight = HashMap::new();
    for &crew in crew_to_possible_flights.keys() {
        let crew_flows_opt = vert_id_to_flows.get(&crew);
        if crew_flows_opt.is_some() {
            let flight_vert_id = crew_flows_opt.unwrap().keys().next().unwrap();
            let flight = vert_id_as_flight(*flight_vert_id);
            crew_to_flight.insert(crew, flight);
        }
    }
    crew_to_flight
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let mut crew_to_possible_flights = HashMap::new();
        crew_to_possible_flights.insert(1, vec![1, 2, 4]);
        crew_to_possible_flights.insert(2, vec![2]);
        crew_to_possible_flights.insert(3, vec![]);

        let crew_to_flight = crew_to_flight(crew_to_possible_flights);

        let crew_1_flight = *crew_to_flight.get(&1).unwrap();
        assert!(crew_1_flight == 1 || crew_1_flight == 4, format!("crew 1 flight is {}", crew_1_flight));
        assert_eq!(crew_to_flight.get(&2), Some(&2));
        assert_eq!(crew_to_flight.get(&3), None);
    }

    #[test]
    fn test2() {
        let mut crew_to_possiblie_flights = HashMap::new();
        crew_to_possiblie_flights.insert(1, vec![1, 2]);
        crew_to_possiblie_flights.insert(2, vec![1]);

        let crew_to_flight = crew_to_flight(crew_to_possiblie_flights);

        assert!(crew_to_flight.get(&1) == Some(&2));
        assert_eq!(crew_to_flight.get(&2), Some(&1));
    }
}