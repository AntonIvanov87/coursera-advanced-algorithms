use w1_flows_in_networks::Edge;
use std::collections::HashSet;
use w1_flows_in_networks::max_flow;
use w1_flows_in_networks::VertId;

type Stock<'a> = &'a[i32];
type Chart<'a> = Vec<Stock<'a>>;

pub fn charts<'a>(stocks: &[Stock<'a>]) -> Vec<Chart<'a>> {

    println!("stocks {:?}", stocks);

    let mut final_charts = Vec::new();
    let charts = stocks.iter().map(|s| vec!(*s)).collect();
    let (mut left_charts, mut right_charts) = split_charts(charts);
    loop {

        println!("left charts {:?}, right charts {:?}, final charts {:?}", left_charts, right_charts, final_charts);

        let source_vert_id = (left_charts.len() + right_charts.len()) as VertId;
        let mut edges: Vec<Edge> = Vec::new();
        for left_chart_i in 0..left_charts.len() {
            let mut need_edge_from_source = false;

            for right_chart_i in 0..right_charts.len() {
                // TODO: need not check for left_charts[0]
                if can_combine_charts(&left_charts[left_chart_i], &right_charts[right_chart_i]) {
                    need_edge_from_source = true;

                    // TODO: reuse logic
                    let right_vert_id = (left_charts.len() + right_chart_i) as u16;
                    edges.push(create_edge(left_chart_i as u16, right_vert_id));
                }
            }

            if need_edge_from_source {
                edges.push(create_edge(source_vert_id, left_chart_i as u16));
            }
        }

        let target_vert_id = source_vert_id + 1;
        for right_chart_i in 0..right_charts.len() {
            // TODO: decopypaste
            let right_vert_id = (left_charts.len() + right_chart_i) as u16;
            edges.push(create_edge(right_vert_id, target_vert_id));
        }

        let vert_id_to_flows = max_flow(&edges, source_vert_id, target_vert_id);

        if vert_id_to_flows.is_empty() {
            // TODO: remove last index, not first
            final_charts.push(left_charts.remove(0));
            left_charts.extend_from_slice(&right_charts);
            if left_charts.is_empty() {
                break;
            }
            let left_and_right_charts = split_charts(left_charts);
            left_charts = left_and_right_charts.0;
            right_charts = left_and_right_charts.1;

        } else if vert_id_to_flows.get(&0).is_none() {
            left_charts.extend_from_slice(&right_charts);
            // TODO: pass base chart explicitly
            let left_and_right_charts = split_charts(left_charts);
            left_charts = left_and_right_charts.0;
            right_charts = left_and_right_charts.1;

        } else {
            let mut new_left_charts = Vec::new();
            let mut combined_right_charts = HashSet::new();
            for left_chart_i in 0..left_charts.len() {
                if let Some(right_vert_id_to_flow) = vert_id_to_flows.get(&(left_chart_i as u16)) {
                    let &right_vert_id = right_vert_id_to_flow.keys().into_iter().next().unwrap();
                    // TODO: encapsulate
                    let right_chart_i = right_vert_id as usize - left_charts.len();

                    // TODO: can we do it without clone?
                    let mut new_chart = left_charts[left_chart_i].clone();

                    let right_chart = &right_charts[right_chart_i];
                    new_chart.extend_from_slice(right_chart);
                    new_left_charts.push(new_chart);

                    combined_right_charts.insert(right_chart_i);
                } else {
                    new_left_charts.push(left_charts[left_chart_i].clone());
                }
            }
            left_charts = new_left_charts;

            right_charts = (0..right_charts.len())
                .filter(|i| !combined_right_charts.contains(i))
                .map(|i| right_charts[i].clone())
                .collect();
        }
    }

    final_charts
}

fn split_charts(mut charts: Vec<Chart>) -> (Vec<Chart>, Vec<Chart>) {
    let mut left_charts = vec![charts.pop().unwrap()];
    let mut right_charts = Vec::new();
    loop {
        if let Some(chart) = charts.pop() {
            if can_combine_charts(&chart, &left_charts[0]) {
                right_charts.push(chart);
            } else {
                left_charts.push(chart);
            }
        } else {
            break;
        }
    }
    (left_charts, right_charts)
}

fn create_edge(from: VertId, to: VertId) -> Edge {
    Edge {
        from,
        to,
        capacity: 1,
    }
}

fn can_combine_charts(chart1: &Chart, chart2: &Chart) -> bool {
    for &stock1 in chart1 {
        for &stock2 in chart2 {
            if !stocks_can_share_chart(stock1, stock2) {
                return false;
            }
        }
    }
    true
}

fn stocks_can_share_chart(stock1: Stock, stock2: Stock) -> bool {

    if stock1[0] == stock2[0] {
        return false;
    }

    let stock_min: &[i32];
    let stock_max: &[i32];
    if stock1[0] < stock2[0] {
        stock_min = stock1;
        stock_max = stock2;
    } else {
        stock_min = stock2;
        stock_max = stock1;
    }

    for i in 1..stock_min.len().min(stock_max.len()) {
        if stock_min[i] >= stock_max[i] {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {

    use super::*;
    use rand;
    use rand::Rng;

    #[test]
    fn test1() {
        let stocks = &[
            &[1, 2, 3, 4][..],
            &[2, 3, 4, 6][..],
            &[6, 5, 4, 3][..]
        ][..];

        let charts = charts(stocks);

        assert_eq!(charts.len(), 2);
    }

    #[test]
    fn test2() {
        let stocks = &[
            &[5,5,5][..],
            &[4,4,6][..],
            &[4,5,4][..]
        ][..];

        let charts = charts(stocks);

        assert_eq!(charts.len(), 3);
    }

    #[test]
    fn test3() {
        let stocks = &[
            &[1][..],
            &[2][..],
            &[3][..]
        ][..];

        let charts = charts(stocks);

        assert_eq!(charts.len(), 1);
    }

    #[test]
    fn test4() {
        let stocks = &[
            &[8, 0][..],
            &[7, 7][..],
            &[8, 8][..],
            &[0, 4][..]
        ][..];

        let charts = charts(stocks);

        assert_eq!(charts.len(), 2);
    }

    #[test]
    fn test5() {
        let stocks = &[
            &[1, 6, 7][..],
            &[0, 0, 0][..],
            &[0, 5, 0][..],
            &[7, 1, 1][..]
        ][..];

        let charts = charts(stocks);

        assert_eq!(charts.len(), 2);
    }

    #[test]
    fn test_rnd() {
        for _ in 0..10000 {
            let stocks = gen_stocks();
            let stocks: Vec<Stock> = stocks.iter().map(|s| &s[..]).collect();

            let charts = charts(&stocks);

            for chart in &charts {
                for stock_i_1 in 0..chart.len()-1 {
                    for stock_i_2 in stock_i_1+1..chart.len() {
                        assert!(stocks_can_share_chart(chart[stock_i_1], chart[stock_i_2]),
                                "stock 1 {:?}, stock 2 {:?}", chart[stock_i_1], chart[stock_i_2]);
                    }
                }
            }

            let less_charts = assign_stocks_to_n_charts(&stocks, charts.len()-1);
            assert!(less_charts.is_none(), "stocks {:?}\nexpected {:?},\nactual {:?}", stocks, less_charts.unwrap(), charts);
        }
    }

    fn gen_stocks() -> Vec<Vec<i32>> {
        let mut rng = rand::thread_rng();
        let stocks_count = rng.gen_range(1, 6);
        let mut stocks: Vec<Vec<i32>> = Vec::with_capacity(stocks_count);
        let points_count = rng.gen_range(1, 26);
        for _ in 0..stocks_count {
            let mut stock: Vec<i32> = Vec::with_capacity(points_count);
            for _ in 0..points_count {
                stock.push(rng.gen_range(0, 9));
            }
            stocks.push(stock);
        }
        stocks
    }

    fn can_add_stock_to_chart(stock: Stock, chart: &Chart) -> bool {
        for stock_from_chart in chart {
            if !stocks_can_share_chart(stock, stock_from_chart) {
                return false;
            }
        }
        true
    }

    fn assign_stocks_to_n_charts<'a>(stocks: &[Stock<'a>], max_charts: usize) -> Option<Vec<Chart<'a>>> {
        let charts = Vec::with_capacity(max_charts);
        assign_stocks_to_charts(stocks, charts, max_charts)
    }

    fn assign_stocks_to_charts<'a>(stocks: &[Stock<'a>], charts: Vec<Chart<'a>>, max_charts: usize) -> Option<Vec<Chart<'a>>> {
        if stocks.len() == 0 {
            return Some(charts);
        }

        for chart_i in 0..charts.len() {
            let chart = &charts[chart_i];
            if can_add_stock_to_chart(stocks[0], chart) {
                let mut chart_with_stock: Chart<'a> = chart.to_vec();
                chart_with_stock.push(stocks[0]);

                let mut charts_with_stock: Vec<Chart<'a>> = charts.clone();
                charts_with_stock[chart_i] = chart_with_stock;

                if let Some(found_charts) = assign_stocks_to_charts(&stocks[1..], charts_with_stock, max_charts) {
                    return Some(found_charts);
                }
            }
        }

        if charts.len() < max_charts {
            let mut new_chart = Vec::new();
            new_chart.push(stocks[0]);

            let mut charts_with_stock = charts.clone();
            charts_with_stock.push(new_chart);

            if let Some(found_charts) = assign_stocks_to_charts(&stocks[1..], charts_with_stock, max_charts) {
                return Some(found_charts);
            }

        }

        return None;
    }

}