use crate::topo::layout::VisualGraph;

// Name tree refers the nested hierarchy of subgraphs and when ordering nodes with functions that has word tree
// this tree is taken into accoutn for detaills see the paper by sander

fn update_col_map(
    vg: &VisualGraph,
    col_map: &mut Vec<usize>,
    layer_idx: usize,
) {
    for (j, n) in vg.dag.ranks()[layer_idx].iter().enumerate() {
        col_map[n.get_index()] = j;
    }
}

pub(crate) fn do_it_inner(vg: &mut VisualGraph, col_map: &mut Vec<usize>) {
    let layers = vg.dag.num_levels();

    // First perform the ordinary barycenter sweeps, but use a sorted nesting
    // traversal after each layer so each subgraph remains contiguous there.
    vg.dag.subgraph_order_by_p_layerwise(0);
    update_col_map(vg, col_map, 0);
    for i in 1..layers {
        vg.dag.node_order_wp_layerwise(i, col_map);
        vg.dag.subgraph_order_by_p_layerwise(i);
        update_col_map(vg, col_map, i);
    }

    let lambda_order = vg.dag.subgraph_order_by_p();
    for row in vg.dag.ranks_mut() {
        row.clear();
    }
    for node in lambda_order {
        let level = vg.dag.level(node);
        vg.dag.ranks_mut()[level].push(node);
    }

    vg.dag.subgraph_order_by_p_layerwise(layers - 1);
    update_col_map(vg, col_map, layers - 1);
    for i in (0..layers - 1).rev() {
        vg.dag.node_order_ws_layerwise(i, col_map);
        vg.dag.subgraph_order_by_p_layerwise(i);
        update_col_map(vg, col_map, i);
    }

    let lambda_order = vg.dag.subgraph_order_by_p();
    for row in vg.dag.ranks_mut() {
        row.clear();
    }
    for node in lambda_order {
        let level = vg.dag.level(node);
        vg.dag.ranks_mut()[level].push(node);
    }
}

fn apply_ordering_graph(vg: &mut VisualGraph, col_map: &mut Vec<usize>) {
    let layers = vg.dag.num_levels();

    // Adjacent leaves now encode Sanders' "is left of" relation. Build the
    // ordering graph once, make it acyclic, and then use a valid topological
    // ordering at every layer. Barycenter weights only break ties between
    // otherwise valid orders.
    vg.dag.order_subgraphs_by_connectivity_seed();
    let ordering_graph = vg.dag.subgraph_ordering_graph();
    let initial_weights = vec![1.; vg.dag.len()];
    vg.dag
        .order_layer_by_ordering_graph(0, &ordering_graph, &initial_weights);
    update_col_map(vg, col_map, 0);
    for i in 1..layers {
        let weights = vg.dag.predecessor_barycenter_weights(col_map);
        vg.dag
            .order_layer_by_ordering_graph(i, &ordering_graph, &weights);
        update_col_map(vg, col_map, i);
    }

    vg.dag.order_layer_by_ordering_graph(
        layers - 1,
        &ordering_graph,
        &initial_weights,
    );
    update_col_map(vg, col_map, layers - 1);
    for i in (0..layers - 1).rev() {
        let weights = vg.dag.successor_barycenter_weights(col_map);
        vg.dag
            .order_layer_by_ordering_graph(i, &ordering_graph, &weights);
        update_col_map(vg, col_map, i);
    }
}

fn num_crossing(
    vg: &VisualGraph,
    a: crate::adt::dag::NodeHandle,
    b: crate::adt::dag::NodeHandle,
    row: &[crate::adt::dag::NodeHandle],
) -> usize {
    let mut sum = 0;
    let mut num_b = 0;

    let a_successors = vg.dag.successors(a);
    let a_predecessors = vg.dag.predecessors(a);
    let b_successors = vg.dag.successors(b);
    let b_predecessors = vg.dag.predecessors(b);

    for node in row {
        let is_a = a_successors.iter().any(|x| x == node)
            || a_predecessors.iter().any(|x| x == node);
        let is_b = b_successors.iter().any(|x| x == node)
            || b_predecessors.iter().any(|x| x == node);
        if is_a {
            sum += num_b;
        }
        if is_b {
            num_b += 1;
        }
    }

    sum
}

fn count_crossings(vg: &VisualGraph) -> usize {
    let layers = vg.dag.num_levels();
    if layers < 2 {
        return 0;
    }

    let mut sum = 0;
    for row_idx in 0..layers - 1 {
        let first = vg.dag.row(row_idx);
        let second = vg.dag.row(row_idx + 1);
        for i in 0..first.len() {
            for j in i + 1..first.len() {
                sum += num_crossing(vg, first[i], first[j], second);
            }
        }
    }
    sum
}

pub fn prepare_borders(vg: &mut VisualGraph) {
    let subgraph_levels = vg.dag.get_subgraph_levels();
    vg.dag.place_horizontal_borders(&subgraph_levels);
    vg.dag.place_vertical_borders();
    // insert Placeholder nodes for borders
    for s in 0..vg.dag.num_subgraphs() {
        let (lvl_start, lvl_end) = subgraph_levels[s];
        vg.add_vertical_border(lvl_start, lvl_end);
    }
}

const SANDERS_ITERATIONS: usize = 8;

#[cfg_attr(not(feature = "log"), allow(unused_assignments, unused_variables))]
pub(crate) fn do_it(vg: &mut VisualGraph) {
    let mut column_map = vec![0; vg.dag.len()];
    for r in vg.dag.ranks().iter() {
        for (j, n) in r.iter().enumerate() {
            column_map[n.get_index()] = j;
        }
    }
    let mut best_ranks = vg.dag.ranks().clone();
    let mut best_crossings = count_crossings(vg);

    for _ in 0..SANDERS_ITERATIONS {
        do_it_inner(vg, &mut column_map);
        let crossings = count_crossings(vg);
        if crossings < best_crossings {
            best_ranks = vg.dag.ranks().clone();
            best_crossings = crossings;
        }
    }
    *vg.dag.ranks_mut() = best_ranks;
    for (layer_idx, _) in vg.dag.ranks().iter().enumerate() {
        update_col_map(vg, &mut column_map, layer_idx);
    }
    apply_ordering_graph(vg, &mut column_map);
    prepare_borders(vg);
}
