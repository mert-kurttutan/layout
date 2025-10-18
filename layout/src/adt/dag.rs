//! This module implements the Ranked-DAG data structure. I's a data structure
//! that represents the edges between nodes in the dag as well as the leveling
//! of the nodes. A rank is the ordering of some nodes along the x-axis. Users
//! of this data structure may change the leveling of nodes, and the only
//! guarantee is that the nodes are assigned to some level.

use std::{cmp, vec};

/// The Ranked-DAG data structure.
#[derive(Debug)]
pub struct DAG {
    /// Nesting Tree for Subgraph Hierarchy
    nesting_tree: NTree,

    /// A list of nodes in the dag.
    nodes: Vec<Node>,

    /// Places nodes in levels.
    ranks: RankType,

    /// levels info
    levels: Vec<usize>,

    /// Perform validation checks.
    validate: bool,

    /// Ranks for top border of subgraphs
    top_border_ranks: Vec<Vec<Vec<SubgraphHandle>>>,

    /// Ranks for bottom border of subgraphs
    bottom_border_ranks: Vec<Vec<Vec<SubgraphHandle>>>,
}

#[derive(Debug)]
struct NTree {
    subgraphs: Vec<Subgraph>,
}

#[derive(Debug)]
struct Subgraph {
    nodes: Vec<NodeHandle>,
    parent_subgraph_idx: SubgraphHandle,
    subgraphs: Vec<SubgraphHandle>,
    left_borders: Vec<NodeHandle>,
    right_borders: Vec<NodeHandle>,
}

#[derive(Debug)]
struct Node {
    // Points to other edges.
    successors: Vec<NodeHandle>,
    predecessors: Vec<NodeHandle>,
    parent_subgraph_idx: SubgraphHandle,
    node_type: NodeType,
}

#[derive(Debug, PartialEq)]
enum NodeType {
    Regular,
    Connector,
    VerticalBorder,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TraverseHandle {
    Node(NodeHandle),
    Subgraph(SubgraphHandle),
}

#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
pub struct SubgraphHandle {
    pub idx: usize,
}

/// Used by users to keep track of nodes that are saved in the DAG.
#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
pub struct NodeHandle {
    idx: usize,
}

pub type RankType = Vec<Vec<NodeHandle>>;

impl NTree {
    pub fn new() -> Self {
        NTree {
            subgraphs: vec![Subgraph::new(SubgraphHandle::new(0))],
        }
    }

    pub(crate) fn new_subgraph(
        &mut self,
        subgraph: Subgraph,
        parent_subgraph_idx: SubgraphHandle,
    ) -> SubgraphHandle {
        let idx = self.subgraphs.len();
        self.subgraphs.push(subgraph);
        if idx == parent_subgraph_idx.get_index() {
            return SubgraphHandle { idx };
        }
        self.subgraphs[parent_subgraph_idx.get_index()]
            .subgraphs
            .push(SubgraphHandle::new(idx));
        SubgraphHandle { idx }
    }
}

impl Subgraph {
    pub(crate) fn new(parent_subgraph_idx: SubgraphHandle) -> Self {
        Subgraph {
            nodes: Vec::new(),
            parent_subgraph_idx,
            subgraphs: Vec::new(),
            left_borders: Vec::new(),
            right_borders: Vec::new(),
        }
    }
}

impl SubgraphHandle {
    pub fn new(idx: usize) -> Self {
        SubgraphHandle { idx }
    }

    pub fn get_index(&self) -> usize {
        self.idx
    }
}

impl NodeHandle {
    pub fn new(x: usize) -> Self {
        NodeHandle { idx: x }
    }
    pub fn get_index(&self) -> usize {
        self.idx
    }
}

impl From<usize> for NodeHandle {
    fn from(idx: usize) -> Self {
        NodeHandle { idx }
    }
}

impl Node {
    pub fn new(
        parent_subgraph_idx: SubgraphHandle,
        node_type: NodeType,
    ) -> Self {
        Node {
            successors: Vec::new(),
            predecessors: Vec::new(),
            parent_subgraph_idx,
            node_type,
        }
    }

    pub fn get_parent_subgraph_index(&self) -> SubgraphHandle {
        self.parent_subgraph_idx
    }
}

/// Node iterator for iterating over nodes in the graph.
#[derive(Debug)]
pub struct NodeIterator {
    curr: usize,
    last: usize,
}

impl Iterator for NodeIterator {
    type Item = NodeHandle;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr == self.last {
            return None;
        }

        let item = Some(NodeHandle::from(self.curr));
        self.curr += 1;
        item
    }
}

enum VerticalBorder {
    Left,
    Right,
}

impl DAG {
    pub fn new() -> Self {
        DAG {
            nesting_tree: NTree::new(),
            nodes: Vec::new(),
            ranks: Vec::new(),
            levels: Vec::new(),
            top_border_ranks: Vec::new(),
            bottom_border_ranks: Vec::new(),
            validate: true,
        }
    }

    pub fn set_validate(&mut self, validate: bool) {
        self.validate = validate;
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.ranks.clear();
        self.levels.clear();
    }

    pub fn iter(&self) -> NodeIterator {
        NodeIterator {
            curr: 0,
            last: self.nodes.len(),
        }
    }

    pub fn add_edge(&mut self, from: NodeHandle, to: NodeHandle) {
        self.nodes[from.idx].successors.push(to);
        self.nodes[to.idx].predecessors.push(from);
    }

    /// Remove an edge from \p from to \p to.
    /// \returns True if an edge was removed.
    pub fn remove_edge(&mut self, from: NodeHandle, to: NodeHandle) -> bool {
        let succ = &mut self.nodes[from.idx].successors;
        let mut removed_succ = false;

        if let Some(pos) = succ.iter().position(|x| *x == to) {
            succ.remove(pos);
            removed_succ = true;
        }

        let pred = &mut self.nodes[to.idx].predecessors;
        let mut removed_pred = false;
        if let Some(pos) = pred.iter().position(|x| *x == from) {
            pred.remove(pos);
            removed_pred = true;
        }

        // We must preserve the invariant that the pred-succ list must always
        // be up to date.
        assert_eq!(removed_pred, removed_succ);
        removed_pred
    }

    /// Create a new subgraph.
    pub(crate) fn new_subgraph(
        &mut self,
        parent_subgraph_idx: SubgraphHandle,
    ) -> SubgraphHandle {
        let subgraph = Subgraph::new(parent_subgraph_idx);
        let subgraph_idx =
            SubgraphHandle::new(self.nesting_tree.subgraphs.len());
        self.nesting_tree
            .new_subgraph(subgraph, parent_subgraph_idx);
        subgraph_idx
    }

    pub(crate) fn subgraph_left_borders(
        &self,
        sg: SubgraphHandle,
    ) -> &Vec<NodeHandle> {
        &self.nesting_tree.subgraphs[sg.idx].left_borders
    }

    pub(crate) fn subgraph_right_borders(
        &self,
        sg: SubgraphHandle,
    ) -> &Vec<NodeHandle> {
        &self.nesting_tree.subgraphs[sg.idx].right_borders
    }

    /// Create a new node.
    pub fn new_node(
        &mut self,
        parent_subgraph_idx: SubgraphHandle,
    ) -> NodeHandle {
        self.nodes
            .push(Node::new(parent_subgraph_idx, NodeType::Regular));
        self.levels.push(0);
        let node = NodeHandle::new(self.nodes.len() - 1);
        self.nesting_tree.subgraphs[parent_subgraph_idx.idx]
            .nodes
            .push(node);
        self.add_element_to_rank(node, 0, false);
        node
    }

    pub(crate) fn new_connector_node(
        &mut self,
        parent_subgraph_idx: SubgraphHandle,
    ) -> NodeHandle {
        self.nodes
            .push(Node::new(parent_subgraph_idx, NodeType::Connector));
        self.levels.push(0);
        let node = NodeHandle::new(self.nodes.len() - 1);
        self.nesting_tree.subgraphs[parent_subgraph_idx.idx]
            .nodes
            .push(node);
        self.add_element_to_rank(node, 0, false);
        node
    }

    /// Create \p n new nodes.
    pub fn new_nodes(&mut self, n: usize) {
        for _ in 0..n {
            self.nodes
                .push(Node::new(SubgraphHandle::new(0), NodeType::Regular));
            self.levels.push(0);
            let node = NodeHandle::new(self.nodes.len() - 1);
            self.add_element_to_rank(node, 0, false);
        }
        self.verify();
    }

    pub fn successors(&self, from: NodeHandle) -> &Vec<NodeHandle> {
        &self.nodes[from.idx].successors
    }

    pub fn predecessors(&self, from: NodeHandle) -> &Vec<NodeHandle> {
        &self.nodes[from.idx].predecessors
    }

    pub fn get_parent_subgraph_index_n(
        &self,
        from: NodeHandle,
    ) -> SubgraphHandle {
        self.nodes[from.idx].get_parent_subgraph_index()
    }

    pub fn get_parent_subgraph_index_sg(
        &self,
        from: SubgraphHandle,
    ) -> SubgraphHandle {
        self.nesting_tree.subgraphs[from.idx].parent_subgraph_idx
    }

    pub fn get_children_subgraphs(
        &self,
        from: SubgraphHandle,
    ) -> &Vec<SubgraphHandle> {
        &self.nesting_tree.subgraphs[from.idx].subgraphs
    }

    pub(crate) fn is_inside_same_subgraph(
        &self,
        from: NodeHandle,
        to: NodeHandle,
    ) -> bool {
        let from_sg = self.get_parent_subgraph_index_n(from);
        let mut cur_sg = self.get_parent_subgraph_index_n(to);
        if cur_sg.get_index() == from_sg.get_index() {
            return true;
        }
        while cur_sg.get_index() != 0 {
            let parent_sg = self.get_parent_subgraph_index_sg(cur_sg);
            if parent_sg.get_index() == from_sg.get_index() {
                return true;
            }
            cur_sg = parent_sg;
        }
        false
    }

    pub fn single_pred(&self, from: NodeHandle) -> Option<NodeHandle> {
        if self.nodes[from.idx].predecessors.len() == 1 {
            return Some(self.nodes[from.idx].predecessors[0]);
        }
        None
    }

    pub fn single_succ(&self, from: NodeHandle) -> Option<NodeHandle> {
        if self.nodes[from.idx].successors.len() == 1 {
            return Some(self.nodes[from.idx].successors[0]);
        }
        None
    }

    pub fn is_vertical_border(&self, node: NodeHandle) -> bool {
        match self.nodes[node.idx].node_type {
            NodeType::VerticalBorder => true,
            _ => false,
        }
    }

    pub fn verify(&self) {
        if self.validate {
            // Check that the node indices are valid.
            for node in &self.nodes {
                for edge in &node.successors {
                    assert!(edge.idx < self.nodes.len());
                }
            }

            // Check that the graph is a DAG.
            for (i, node) in self.nodes.iter().enumerate() {
                let from = NodeHandle::from(i);
                for dest in node.successors.iter() {
                    let reachable =
                        self.is_reachable(*dest, from) && from != *dest;
                    assert!(!reachable, "We found a cycle!");
                }
            }

            // Make sure that all of the nodes are in ranks.
            assert_eq!(self.count_nodes_in_ranks(), self.len());
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub(crate) fn get_subgraph_levels(&self) -> Vec<(usize, usize)> {
        let mut levels = vec![(usize::MAX, usize::MIN); self.num_subgraphs()];
        for (i, node) in self.nodes.iter().enumerate() {
            let subgraph_idx = node.get_parent_subgraph_index();
            let level = self.level(NodeHandle::from(i));
            levels[subgraph_idx.idx].0 =
                cmp::min(levels[subgraph_idx.idx].0, level);
            levels[subgraph_idx.idx].1 =
                cmp::max(levels[subgraph_idx.idx].1, level);
        }

        for (i, s) in self.nesting_tree.subgraphs.iter().enumerate().rev() {
            for child in s.subgraphs.iter() {
                levels[i].0 = cmp::min(levels[i].0, levels[child.idx].0);
                levels[i].1 = cmp::max(levels[i].1, levels[child.idx].1);
            }
        }
        levels
    }
    /// \returns True if the node \to is reachable from the node \p from.
    /// This internal method is used for the verification of the graph.
    fn is_reachable_inner(
        &self,
        from: NodeHandle,
        to: NodeHandle,
        visited: &mut Vec<bool>,
    ) -> bool {
        if from == to {
            return true;
        }

        // Don't step into a cycle.
        if visited[from.idx] {
            return false;
        }

        // Push to the dfs stack.
        visited[from.idx] = true;

        let from_node = &self.nodes[from.idx];
        for edge in &from_node.successors {
            if self.is_reachable_inner(*edge, to, visited) {
                return true;
            }
        }

        // Pop from the dfs stack.
        visited[from.idx] = false;
        false
    }

    /// \returns True if there is a path from \p 'from' to \p 'to'.
    pub fn is_reachable(&self, from: NodeHandle, to: NodeHandle) -> bool {
        if from == to {
            return true;
        }

        let mut visited = Vec::new();
        visited.resize(self.nodes.len(), false);
        self.is_reachable_inner(from, to, &mut visited)
    }

    fn topological_sort_traverse_inner(
        &self,
        subgraph_idx: SubgraphHandle,
        node_successors_t: &Vec<Vec<TraverseHandle>>,
        subgraph_successors_t: &Vec<Vec<TraverseHandle>>,
    ) -> Vec<TraverseHandle> {
        // A list of vectors in post-order.
        let mut order: Vec<TraverseHandle> = Vec::new();

        // Marks that a node is in the worklist.
        let mut visited_nodes = vec![false; self.nodes.len()];
        let mut visited_subgraphs =
            vec![false; self.nesting_tree.subgraphs.len()];

        // A tuple of handle, and command:
        // true- force push.
        // false- this is a child to visit.
        let mut worklist: Vec<(TraverseHandle, bool)> = Vec::new();

        // Add all of the values that we want to compute into the worklist.
        for n in self.nesting_tree.subgraphs[subgraph_idx.idx].nodes.iter() {
            worklist.push((TraverseHandle::Node(*n), false));
        }
        for s in self.nesting_tree.subgraphs[subgraph_idx.idx]
            .subgraphs
            .iter()
        {
            worklist.push((TraverseHandle::Subgraph(*s), false));
        }

        while let Some((current, cmd)) = worklist.pop() {
            // Handle 'push' commands.
            if cmd {
                order.push(current);
                continue;
            }

            match current {
                TraverseHandle::Node(n) => {
                    // Don't visit visited nodes.
                    if visited_nodes[n.idx] {
                        continue;
                    }

                    visited_nodes[n.idx] = true;

                    // Save this node after all of the children are handles.
                    worklist.push((current, true));

                    // Add the children to the worklist.
                    let succs = node_successors_t[n.idx].clone();
                    // succs.reverse();
                    for edge in succs.iter() {
                        worklist.push((edge.clone(), false));
                    }
                }
                TraverseHandle::Subgraph(s) => {
                    if visited_subgraphs[s.idx] {
                        continue;
                    }
                    visited_subgraphs[s.idx] = true;
                    worklist.push((current, true));
                    let succs = subgraph_successors_t[s.idx].clone();
                    for edge in succs.iter() {
                        worklist.push((edge.clone(), false));
                    }
                }
            }
        }

        order.reverse();
        order
    }

    fn compute_successors_t(
        &self,
    ) -> (Vec<Vec<TraverseHandle>>, Vec<Vec<TraverseHandle>>) {
        let mut node_predecessors_t = vec![Vec::new(); self.nodes.len()];
        let mut subgraph_predecessors_t =
            vec![Vec::new(); self.nesting_tree.subgraphs.len()];

        for (i, node) in self.nodes.iter().enumerate() {
            for succ in node.successors.iter() {
                let (from_traverse, to_traverse) =
                    self.nesting_edge_pair(NodeHandle::new(i), *succ);
                match from_traverse {
                    TraverseHandle::Node(n) => {
                        node_predecessors_t[n.idx].push(to_traverse);
                    }
                    TraverseHandle::Subgraph(s) => {
                        subgraph_predecessors_t[s.idx].push(to_traverse);
                    }
                }
            }
        }
        (node_predecessors_t, subgraph_predecessors_t)
    }
    fn compute_predecessors_t(
        &self,
    ) -> (Vec<Vec<TraverseHandle>>, Vec<Vec<TraverseHandle>>) {
        let mut node_predecessors_t = vec![Vec::new(); self.nodes.len()];
        let mut subgraph_predecessors_t =
            vec![Vec::new(); self.nesting_tree.subgraphs.len()];

        for (i, node) in self.nodes.iter().enumerate() {
            for pred in node.predecessors.iter() {
                let (from_traverse, to_traverse) =
                    self.nesting_edge_pair(*pred, NodeHandle::new(i));
                match to_traverse {
                    TraverseHandle::Node(n) => {
                        node_predecessors_t[n.idx].push(from_traverse);
                    }
                    TraverseHandle::Subgraph(s) => {
                        subgraph_predecessors_t[s.idx].push(from_traverse);
                    }
                }
            }
        }
        (node_predecessors_t, subgraph_predecessors_t)
    }

    pub fn topological_sort_traverse(&self) -> Vec<NodeHandle> {
        let (node_successors_t, subgraph_successors_t) =
            self.compute_successors_t();
        let subgraphs_num = self.num_subgraphs();
        let mut traverse_orders = vec![vec![]; subgraphs_num];
        for i in (0..subgraphs_num).rev() {
            let order_t = self.topological_sort_traverse_inner(
                SubgraphHandle::new(i),
                &node_successors_t,
                &subgraph_successors_t,
            );
            let mut order_t_expanded = Vec::new();
            for t in order_t {
                match t {
                    TraverseHandle::Node(n) => order_t_expanded.push(n),
                    TraverseHandle::Subgraph(s) => {
                        order_t_expanded.extend(traverse_orders[s.idx].iter());
                    }
                }
            }
            traverse_orders[i] = order_t_expanded;
        }
        traverse_orders[0].clone()
    }

    // The methods below are related to the rank (placing nodes in levels). //

    /// \returns the number of ranks in the dag.
    pub fn num_levels(&self) -> usize {
        self.ranks.len()
    }

    pub fn num_subgraphs(&self) -> usize {
        self.nesting_tree.subgraphs.len()
    }

    /// \return a mutable reference to a row at level \p level.
    pub fn row_mut(&mut self, level: usize) -> &mut Vec<NodeHandle> {
        assert!(level < self.ranks.len(), "Invalid rank");
        &mut self.ranks[level]
    }

    /// \return a reference to a row at level \p level.
    pub fn row(&self, level: usize) -> &Vec<NodeHandle> {
        assert!(level < self.ranks.len(), "Invalid rank");
        &self.ranks[level]
    }

    /// \return a reference to the whole rank data structure.
    pub fn ranks(&self) -> &RankType {
        &self.ranks
    }

    /// \return a mutable reference to the whole rank data structure.
    pub fn ranks_mut(&mut self) -> &mut RankType {
        &mut self.ranks
    }

    /// \returns a reference to the whole border ranks data structure.
    pub(crate) fn top_border_ranks(&self) -> &Vec<Vec<Vec<SubgraphHandle>>> {
        &self.top_border_ranks
    }

    /// \returns a reference to the whole border ranks data structure.
    pub(crate) fn bottom_border_ranks(&self) -> &Vec<Vec<Vec<SubgraphHandle>>> {
        &self.bottom_border_ranks
    }

    /// \returns True if \p elem is the first node in the row \p level.
    pub fn is_first_in_row(&self, elem: NodeHandle, level: usize) -> bool {
        if level >= self.ranks.len() || self.ranks[level].is_empty() {
            return false;
        }
        self.ranks[level][0] == elem
    }

    /// \returns True if \p elem is the last node in the row \p level.
    pub fn is_last_in_row(&self, elem: NodeHandle, level: usize) -> bool {
        if level >= self.ranks.len() || self.ranks[level].is_empty() {
            return false;
        }
        let last_idx = self.ranks[level].len() - 1;
        self.ranks[level][last_idx] == elem
    }

    /// Place the element \p elem at the nth level \p level. If the level does
    /// not exist then create it. If \p prepend is set then the node is inserted
    /// at the beginning of the rank. The node must not be in the rank when this
    /// method is called.
    fn add_element_to_rank(
        &mut self,
        elem: NodeHandle,
        level: usize,
        prepend: bool,
    ) {
        while self.ranks.len() < level + 1 {
            self.ranks.push(Vec::new());
        }

        if prepend {
            self.ranks[level].insert(0, elem);
        } else {
            self.ranks[level].push(elem);
        }
        self.levels[elem.get_index()] = level;
    }

    /// Places all of the nodes in ranks (levels).
    pub fn recompute_node_ranks(&mut self) {
        assert!(!self.is_empty(), "Sorting an empty graph");
        let order = self.topological_sort_traverse();
        let mut levels = self.compute_levels(&order);
        self.compactify_subgraph(&order, &mut levels);
        self.ranks.clear();
        for (i, level) in levels.iter().enumerate() {
            self.add_element_to_rank(NodeHandle::from(i), *level, false);
        }
    }

    /// \returns the number of nodes that are in ranks.
    /// This is used for verification of the dag.
    fn count_nodes_in_ranks(&self) -> usize {
        let mut cnt = 0;
        for row in self.ranks.iter() {
            cnt += row.len();
        }
        cnt
    }

    /// Move the node \p node to a new level \p new_level.
    /// Place the node before \p node, or at the end.
    pub fn update_node_rank_level(
        &mut self,
        node: NodeHandle,
        new_level: usize,
        insert_before: Option<NodeHandle>,
    ) {
        let curr_level = self.level(node);
        let level = &mut self.ranks[curr_level];
        let idx = level
            .iter()
            .position(|x| *x == node)
            .expect("node not found");
        level.remove(idx);

        // Make sure that the row exists.
        while self.ranks.len() < new_level + 1 {
            self.ranks.push(Vec::new());
        }

        if let Option::Some(marker) = insert_before {
            let row = &mut self.ranks[new_level];
            for i in 0..row.len() {
                if row[i] == marker {
                    row.insert(i, node);
                    self.levels[node.get_index()] = new_level;
                    return;
                }
            }
            panic!("Can't find the marker node in the array");
        }

        self.ranks[new_level].push(node);
        self.levels[node.get_index()] = new_level;
        assert_eq!(self.level(node), new_level);
    }

    /// \returns the level of the node \p node in the rank.
    pub fn level(&self, node: NodeHandle) -> usize {
        assert!(node.get_index() < self.len(), "Node not in the dag");
        self.levels[node.get_index()]
    }

    /// Computes and returns the level of each node in the graph based
    /// on the traversal order \p order.
    fn compute_levels(&self, order: &[NodeHandle]) -> Vec<usize> {
        let mut levels: Vec<usize> = Vec::new();
        assert_eq!(order.len(), self.nodes.len());

        // Levels has the same layout as the DAG node list.
        levels.resize(self.nodes.len(), 0);

        // For each node in the order (starting with a node of level zero).
        for src in order {
            // Update the level of all successors.
            for dest in self.nodes[src.idx].successors.iter() {
                // Ignore self edges.
                if src.idx == dest.idx {
                    continue;
                }
                levels[dest.idx] =
                    cmp::max(levels[dest.idx], levels[src.idx] + 1);
            }
        }

        levels
    }

    pub(crate) fn place_horizontal_borders(
        &mut self,
        subgraph_levels: &Vec<(usize, usize)>,
    ) {
        let num_levels = self.num_levels();
        self.top_border_ranks = vec![vec![]; num_levels];
        self.bottom_border_ranks = vec![vec![]; num_levels];

        self.top_border_ranks[0] = vec![vec![SubgraphHandle::new(0)]];
        self.bottom_border_ranks[num_levels - 1] =
            vec![vec![SubgraphHandle::new(0)]];

        let mut inner_levels = vec![None; self.num_subgraphs()];

        inner_levels[0] = Some((0, 0));
        for s in 0..self.num_subgraphs() {
            self.new_horizontal_border(
                SubgraphHandle::new(s),
                subgraph_levels,
                &mut inner_levels,
            );
        }
    }

    fn new_horizontal_border(
        &mut self,
        subgraph_idx: SubgraphHandle,
        levels_map: &Vec<(usize, usize)>,
        inner_levels: &mut Vec<Option<(usize, usize)>>,
    ) {
        let (p_lvl_s, p_lvl_e) = levels_map[subgraph_idx.idx];

        let (p_in_lvl_s, p_in_lvl_e) = inner_levels[subgraph_idx.idx].expect(
            "Ordering subgraph horizontal borders is wrong, this is a bug.",
        );

        let mut c_in_lvl_s;
        let mut c_in_lvl_e;

        for child_s in self.nesting_tree.subgraphs[subgraph_idx.idx]
            .subgraphs
            .iter()
        {
            let (lvl_s, lvl_e) = levels_map[child_s.idx];
            if lvl_s == p_lvl_s {
                if p_in_lvl_s + 1 < self.top_border_ranks[lvl_s].len() {
                    self.top_border_ranks[lvl_s][p_in_lvl_s + 1].push(*child_s);
                } else {
                    self.top_border_ranks[lvl_s].push(vec![*child_s]);
                }

                c_in_lvl_s = p_in_lvl_s + 1;
            } else {
                if self.top_border_ranks[lvl_s].is_empty() {
                    self.top_border_ranks[lvl_s].push(vec![*child_s]);
                    c_in_lvl_s = 0;
                } else {
                    self.top_border_ranks[lvl_s]
                        .last_mut()
                        .unwrap()
                        .push(*child_s);
                    c_in_lvl_s = self.top_border_ranks[lvl_s].len() - 1;
                }
            }

            if lvl_e == p_lvl_e {
                if p_in_lvl_e + 1 < self.top_border_ranks[lvl_e].len() {
                    self.top_border_ranks[lvl_e][p_in_lvl_e + 1].push(*child_s);
                } else {
                    self.bottom_border_ranks[lvl_e].push(vec![*child_s]);
                }
                c_in_lvl_e = p_in_lvl_e + 1;
            } else {
                if self.bottom_border_ranks[lvl_e].is_empty() {
                    self.bottom_border_ranks[lvl_e].push(vec![*child_s]);
                    c_in_lvl_e = 0;
                } else {
                    self.bottom_border_ranks[lvl_e]
                        .last_mut()
                        .unwrap()
                        .push(*child_s);
                    c_in_lvl_e = self.bottom_border_ranks[lvl_e].len() - 1;
                }
            }

            inner_levels[child_s.idx] = Some((c_in_lvl_s, c_in_lvl_e));
        }
    }

    fn get_tree_path(
        &self,
        start_sg: SubgraphHandle,
        end_sg: SubgraphHandle,
    ) -> (Vec<SubgraphHandle>, SubgraphHandle, Vec<SubgraphHandle>) {
        let from_path = self.subgraph_path(start_sg);
        let to_path = self.subgraph_path(end_sg);
        let max_len = from_path.len().min(to_path.len());
        let mut i = 0;
        while i < max_len {
            if from_path[i] == to_path[i] {
                i += 1;
            } else {
                break;
            }
        }
        let from_path = from_path.iter().skip(i).rev().cloned().collect();
        (from_path, to_path[i - 1], to_path[i..].to_vec())
    }

    fn new_vertical_border_node(
        &mut self,
        sg: SubgraphHandle,
        level: usize,
        border_type: VerticalBorder,
    ) -> NodeHandle {
        self.nodes.push(Node::new(sg, NodeType::VerticalBorder));
        let node_handle = NodeHandle::new(self.nodes.len() - 1);
        self.nesting_tree.subgraphs[sg.idx].nodes.push(node_handle);
        self.levels.push(level);
        self.ranks[level].push(node_handle);
        match border_type {
            VerticalBorder::Left => self.nesting_tree.subgraphs[sg.idx]
                .left_borders
                .push(node_handle),
            VerticalBorder::Right => self.nesting_tree.subgraphs[sg.idx]
                .right_borders
                .push(node_handle),
        }
        node_handle
    }

    pub(crate) fn place_vertical_borders(&mut self) {
        let max_level = self.num_levels();
        for ri in 0..max_level {
            let row_len = self.ranks[ri].len();
            let old_row = self.ranks[ri].clone();
            self.ranks[ri].clear();
            let sg_path = self
                .subgraph_path(self.get_parent_subgraph_index_n(old_row[0]));
            for s in sg_path.iter() {
                self.new_vertical_border_node(*s, ri, VerticalBorder::Left);
            }
            for i in 0..(row_len - 1) {
                self.ranks[ri].push(old_row[i]);
                let curr_sg = self.get_parent_subgraph_index_n(old_row[i]);
                let next_sg = self.get_parent_subgraph_index_n(old_row[i + 1]);
                if curr_sg == next_sg {
                    continue;
                }
                let (from_path, _, to_path) =
                    self.get_tree_path(curr_sg, next_sg);
                for s in from_path.iter() {
                    self.new_vertical_border_node(
                        *s,
                        ri,
                        VerticalBorder::Right,
                    );
                }

                for s in to_path.iter() {
                    self.new_vertical_border_node(*s, ri, VerticalBorder::Left);
                }
            }
            self.ranks[ri].push(old_row[row_len - 1]);
            let sg_path = self.subgraph_path(
                self.get_parent_subgraph_index_n(old_row[row_len - 1]),
            );
            for s in sg_path.iter().rev() {
                self.new_vertical_border_node(*s, ri, VerticalBorder::Right);
            }

            // go trhough left and right borders and connect them
            for s in 0..self.num_subgraphs() {
                let border_len =
                    self.nesting_tree.subgraphs[s].right_borders.len();
                for i in 1..border_len {
                    self.add_edge(
                        self.nesting_tree.subgraphs[s].left_borders[i - 1],
                        self.nesting_tree.subgraphs[s].left_borders[i],
                    );
                    self.add_edge(
                        self.nesting_tree.subgraphs[s].right_borders[i - 1],
                        self.nesting_tree.subgraphs[s].right_borders[i],
                    );
                }
            }
        }
        // assert right and left border length are equal for each subgraph
        for s in 0..self.num_subgraphs() {
            assert_eq!(
                self.nesting_tree.subgraphs[s].left_borders.len(),
                self.nesting_tree.subgraphs[s].right_borders.len(),
                "Subgraph {:?} has unbalanced borders",
                SubgraphHandle::new(s)
            );
        }
    }

    fn barycenter_weight_pred(
        &self,
        node: NodeHandle,
        col_map: &Vec<usize>,
    ) -> f64 {
        let mut total_weight = 0;
        let total_count = self.predecessors(node).len();
        for p in self.predecessors(node).iter() {
            total_weight += col_map[p.get_index()];
        }

        if total_count == 0 {
            return col_map[node.get_index()] as f64;
        }
        total_weight as f64 / total_count as f64
    }

    fn barycenter_weight_succ(
        &self,
        node: NodeHandle,
        col_map: &Vec<usize>,
    ) -> f64 {
        let mut total_weight = 0;
        let total_count = self.successors(node).len();
        for p in self.successors(node).iter() {
            total_weight += col_map[p.get_index()];
        }

        if total_count == 0 {
            return col_map[node.get_index()] as f64;
        }
        total_weight as f64 / total_count as f64
    }

    fn aggregate_position(&self) -> (Vec<f64>, Vec<f64>) {
        let subgraphs_num = self.num_subgraphs();
        let mut total_position = vec![(0, 0); subgraphs_num];
        let mut position_nodes = vec![0.0; self.len()];

        for row in self.ranks.iter() {
            for (j, node) in row.iter().enumerate() {
                let subgraph_idx =
                    self.nodes[node.idx].get_parent_subgraph_index();
                total_position[subgraph_idx.idx].0 += j;
                total_position[subgraph_idx.idx].1 += 1;
                position_nodes[node.idx] = j as f64;
            }
        }
        for i in (0..subgraphs_num).rev() {
            for j in self.nesting_tree.subgraphs[i].subgraphs.iter() {
                total_position[i].0 += total_position[j.idx].0;
                total_position[i].1 += total_position[j.idx].1;
            }
        }

        let average_position: Vec<f64> = total_position
            .into_iter()
            .map(|(total, count)| {
                if count == 0 {
                    0.0
                } else {
                    total as f64 / count as f64
                }
            })
            .collect();

        (average_position, position_nodes)
    }

    fn aggregate_position_layerwise(
        &self,
        layer_idx: usize,
    ) -> (Vec<Option<f64>>, Vec<Option<f64>>) {
        let subgraphs_num = self.num_subgraphs();

        let mut total_position = vec![(0, 0); subgraphs_num];
        let mut position_nodes = vec![None; self.len()];
        for (j, node) in self.ranks[layer_idx].iter().enumerate() {
            let subgraph_idx = self.nodes[node.idx].get_parent_subgraph_index();
            total_position[subgraph_idx.idx].0 += j;
            total_position[subgraph_idx.idx].1 += 1;
            position_nodes[node.idx] = Some(j as f64);
        }

        for i in 0..subgraphs_num {
            for j in self.nesting_tree.subgraphs[subgraphs_num - i - 1]
                .subgraphs
                .iter()
            {
                total_position[subgraphs_num - i - 1].0 +=
                    total_position[j.idx].0;
                total_position[subgraphs_num - i - 1].1 +=
                    total_position[j.idx].1;
            }
        }

        let average_position: Vec<Option<f64>> = total_position
            .into_iter()
            .map(|(total, count)| {
                if count == 0 {
                    None
                } else {
                    Some(total as f64 / count as f64)
                }
            })
            .collect();

        (average_position, position_nodes)
    }

    pub(crate) fn subgraph_order_by_p_layerwise(&mut self, layer_idx: usize) {
        let (subgraph_pos_map, node_pos) =
            self.aggregate_position_layerwise(layer_idx);

        let mut working_list =
            vec![TraverseHandle::Subgraph(SubgraphHandle::new(0))];
        let mut output = vec![];

        while !working_list.is_empty() {
            let current = working_list.pop().unwrap();
            match current {
                TraverseHandle::Node(n) => {
                    output.push(n);
                }
                TraverseHandle::Subgraph(s) => {
                    let mut cur = vec![];
                    for s_child in
                        self.nesting_tree.subgraphs[s.idx].subgraphs.iter()
                    {
                        if let Some(_) = subgraph_pos_map[s_child.idx] {
                            cur.push(TraverseHandle::Subgraph(*s_child));
                        }
                    }

                    for n_child in
                        self.nesting_tree.subgraphs[s.idx].nodes.iter()
                    {
                        if let Some(_) = node_pos[n_child.idx] {
                            cur.push(TraverseHandle::Node(*n_child));
                        }
                    }

                    cur.sort_by(|a, b| {
                        let p_a = match a {
                            TraverseHandle::Node(n) => node_pos[n.idx].unwrap(),
                            TraverseHandle::Subgraph(sg) => {
                                subgraph_pos_map[sg.idx].unwrap()
                            }
                        };
                        let p_b = match b {
                            TraverseHandle::Node(n) => node_pos[n.idx].unwrap(),
                            TraverseHandle::Subgraph(sg) => {
                                subgraph_pos_map[sg.idx].unwrap()
                            }
                        };
                        p_b.partial_cmp(&p_a).unwrap()
                    });

                    for c in cur.into_iter() {
                        working_list.push(c);
                    }
                }
            }
        }

        self.ranks[layer_idx] = output;
    }

    pub(crate) fn subgraph_order_by_p(&mut self) -> Vec<NodeHandle> {
        let (subgraph_pos_map, node_pos) = self.aggregate_position();
        let mut working_list =
            vec![TraverseHandle::Subgraph(SubgraphHandle::new(0))];
        let mut output = vec![];

        while !working_list.is_empty() {
            let current = working_list.pop().unwrap();
            match current {
                TraverseHandle::Node(n) => {
                    output.push(n);
                }
                TraverseHandle::Subgraph(s) => {
                    let mut cur = vec![];
                    let mut connectors = vec![];
                    for s_child in
                        self.nesting_tree.subgraphs[s.idx].subgraphs.iter()
                    {
                        cur.push(TraverseHandle::Subgraph(*s_child));
                    }
                    for n_child in
                        self.nesting_tree.subgraphs[s.idx].nodes.iter()
                    {
                        match self.nodes[n_child.idx].node_type {
                            NodeType::Connector => {
                                let from = self.single_pred(*n_child).unwrap();
                                let to = self.single_succ(*n_child).unwrap();
                                let x = self.nesting_edge_pair(from, to);
                                connectors.push((x, *n_child));
                            }
                            _ => {
                                cur.push(TraverseHandle::Node(*n_child));
                            }
                        }
                    }

                    cur.sort_by(|a, b| {
                        let p_a = match a {
                            TraverseHandle::Node(n) => node_pos[n.idx],
                            TraverseHandle::Subgraph(sg) => {
                                subgraph_pos_map[sg.idx]
                            }
                        };
                        let p_b = match b {
                            TraverseHandle::Node(n) => node_pos[n.idx],
                            TraverseHandle::Subgraph(sg) => {
                                subgraph_pos_map[sg.idx]
                            }
                        };
                        p_b.partial_cmp(&p_a).unwrap()
                    });

                    connectors.sort_by(|a, b| a.1.idx.cmp(&b.1.idx));

                    // insert connectors
                    for c in connectors.iter() {
                        let conn_1st_pos =
                            cur.iter().position(|x| *x == c.0 .0).unwrap();
                        cur.insert(conn_1st_pos, TraverseHandle::Node(c.1));
                    }

                    for c in cur.into_iter() {
                        working_list.push(c);
                    }
                }
            }
        }
        output
    }

    pub(crate) fn node_order_ws_layerwise(
        &mut self,
        layer_idx: usize,
        col_map: &Vec<usize>,
    ) {
        // using barycenter weights to order nodes in the layer
        let mut result = Vec::new();
        let mut node_weights = Vec::new();
        for node in self.row(layer_idx).iter() {
            let weight = self.barycenter_weight_succ(*node, col_map);
            node_weights.push((weight, *node));
        }
        // sort by weight
        node_weights.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        for (_, node) in node_weights.iter() {
            result.push(*node);
        }
        self.ranks[layer_idx] = result;
    }

    pub(crate) fn node_order_wp_layerwise(
        &mut self,
        layer_idx: usize,
        col_map: &Vec<usize>,
    ) {
        // using barycenter weights to order nodes in the layer
        let mut result = Vec::new();
        let mut node_weights = Vec::new();
        for node in self.row(layer_idx).iter() {
            let weight = self.barycenter_weight_pred(*node, col_map);
            node_weights.push((weight, *node));
        }
        // sort by weight
        node_weights.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        for (_, node) in node_weights.iter() {
            result.push(*node);
        }
        self.ranks[layer_idx] = result;
    }

    fn subgraph_path(&self, sg: SubgraphHandle) -> Vec<SubgraphHandle> {
        let mut path = vec![sg];
        let mut current = sg;
        while current.idx != 0 {
            current =
                self.nesting_tree.subgraphs[current.idx].parent_subgraph_idx;
            path.push(current);
        }
        path.reverse();
        path
    }

    pub(crate) fn nesting_edge_pair(
        &self,
        from: NodeHandle,
        to: NodeHandle,
    ) -> (TraverseHandle, TraverseHandle) {
        let from_path =
            self.subgraph_path(self.get_parent_subgraph_index_n(from));
        let to_path = self.subgraph_path(self.get_parent_subgraph_index_n(to));
        let max_len = from_path.len().min(to_path.len());
        let mut i = 0;
        while i < max_len {
            if from_path[i] != to_path[i] {
                break;
            }
            i += 1;
        }
        let from_t = if i == from_path.len() {
            TraverseHandle::Node(from)
        } else {
            TraverseHandle::Subgraph(from_path[i])
        };

        let to_t = if i == to_path.len() {
            TraverseHandle::Node(to)
        } else {
            TraverseHandle::Subgraph(to_path[i])
        };
        (from_t, to_t)
    }

    fn compactify_subgraph(
        &self,
        order: &[NodeHandle],
        levels: &mut Vec<usize>,
    ) {
        let (node_predecessors_t, subgraph_predecessors_t) =
            self.compute_predecessors_t();
        let subgraphs_num = self.nesting_tree.subgraphs.len();
        // go through each subgraph and record the max level of its nodes with in subgraph predesosor that is also not connector
        let mut subgraph_max_root_level = vec![0; subgraphs_num];
        let mut subgraph_roots = vec![vec![]; subgraphs_num];
        for (i, node) in self.nodes.iter().enumerate() {
            // if node is a connector, skip
            match node.node_type {
                NodeType::Connector => {
                    continue;
                }
                _ => {}
            }
            let subgraph_idx = node.get_parent_subgraph_index();
            // let is_insubgraph_root = node.predecessors_t.is_empty();
            let is_insubgraph_root = node_predecessors_t[i].is_empty();
            if !is_insubgraph_root {
                continue;
            }
            for pred in node.predecessors.iter() {
                let pred_subgraph_idx =
                    self.nodes[pred.idx].get_parent_subgraph_index();

                if subgraph_idx != pred_subgraph_idx {
                    subgraph_max_root_level[subgraph_idx.idx] = cmp::max(
                        subgraph_max_root_level[subgraph_idx.idx],
                        levels[i],
                    );
                    subgraph_roots[subgraph_idx.idx].push(NodeHandle::new(i));
                }
            }
            if node.predecessors.is_empty() {
                subgraph_roots[subgraph_idx.idx].push(NodeHandle::new(i));
                subgraph_max_root_level[subgraph_idx.idx] = cmp::max(
                    subgraph_max_root_level[subgraph_idx.idx],
                    levels[i],
                );
            }
        }
        for i in 0..subgraphs_num - 1 {
            let cur_idx = subgraphs_num - i - 1;
            let cur_subgraph = &self.nesting_tree.subgraphs[cur_idx];
            let parent_idx = cur_subgraph.parent_subgraph_idx.idx;
            let is_subgraph_root = subgraph_predecessors_t[cur_idx].is_empty();
            if is_subgraph_root {
                subgraph_max_root_level[parent_idx] = cmp::max(
                    subgraph_max_root_level[parent_idx],
                    subgraph_max_root_level[cur_idx],
                );
                subgraph_max_root_level[cur_idx] =
                    subgraph_max_root_level[parent_idx];
                for r in subgraph_roots[cur_idx].clone() {
                    subgraph_roots[parent_idx].push(r);
                }
            }
        }

        // now create a map from nodehandle idx to its accompanying subgraph root nodehandles (vec of nodehandles)
        let mut subgraph_root_map = vec![vec![]; self.nodes.len()];
        for root_nodes in subgraph_roots.iter().rev() {
            for n in root_nodes {
                let mut root_list = root_nodes.clone();
                root_list.retain(|&x| x != *n);
                subgraph_root_map[n.idx] = root_list;
            }
        }

        let mut order_c = vec![];
        for src in order {
            if order_c.contains(src) {
                continue;
            }
            order_c.push(*src);
            for accompanying_root in subgraph_root_map[src.idx].iter() {
                order_c.push(*accompanying_root);
            }
        }
        let mut visited = vec![false; self.nodes.len()];

        // For each node in the order (starting with a node of level zero).
        for src in order_c.iter() {
            for accompanying_root in subgraph_root_map[src.idx].iter() {
                levels[src.idx] =
                    cmp::max(levels[accompanying_root.idx], levels[src.idx]);
            }
            for accompanying_root in subgraph_root_map[src.idx].iter() {
                levels[accompanying_root.idx] = levels[src.idx];
            }
            // Update the level of all successors.
            for dest in self.nodes[src.idx].successors.iter() {
                // Ignore self edges.
                if src.idx == dest.idx {
                    continue;
                }
                // pass if dest is before src in order_c
                if visited[dest.idx] {
                    continue;
                }

                levels[dest.idx] =
                    cmp::max(levels[dest.idx], levels[src.idx] + 1);
                // now to keep invariant of root subingraph nodes, if dest is a root node, update all its accompanying root nodes
                for accompanying_root in subgraph_root_map[dest.idx].iter() {
                    levels[dest.idx] = cmp::max(
                        levels[accompanying_root.idx],
                        levels[dest.idx],
                    );
                }
                for accompanying_root in subgraph_root_map[dest.idx].iter() {
                    levels[accompanying_root.idx] = levels[dest.idx];
                }
            }
            visited[src.idx] = true;
        }
    }
}

impl Default for DAG {
    fn default() -> Self {
        Self::new()
    }
}

#[test]
fn test_simple_construction() {
    let mut g = DAG::new();
    let subgraph_idx = SubgraphHandle::new(0);
    let h0 = g.new_node(subgraph_idx);
    g.verify();

    let h1 = g.new_node(subgraph_idx);
    let h2 = g.new_node(subgraph_idx);
    let h3 = g.new_node(subgraph_idx);
    let h4 = g.new_node(subgraph_idx);

    assert_ne!(h0, h1);
    assert_ne!(h1, h2);

    g.add_edge(h0, h1);
    g.add_edge(h1, h2);
    g.add_edge(h0, h2);
    g.add_edge(h2, h3);
    g.add_edge(h3, h4);

    g.verify();

    let order = g.topological_sort_traverse();
    let levels = g.compute_levels(&order);
    assert_eq!(order.len(), g.len());
    assert_eq!(levels.len(), g.len());

    for i in 0..g.len() {
        println!("{}) node {},  level {}", i, order[i].idx, levels[i]);
    }
}

#[test]
fn test_rank_api() {
    let mut g = DAG::new();
    let subgraph_idx = SubgraphHandle::new(0);
    let h0 = g.new_node(subgraph_idx);
    let h1 = g.new_node(subgraph_idx);
    let h2 = g.new_node(subgraph_idx);

    g.add_edge(h0, h1);
    g.add_edge(h1, h2);

    g.recompute_node_ranks();
    g.verify();

    assert_eq!(g.level(h0), 0);
    assert_eq!(g.level(h1), 1);
    assert_eq!(g.level(h2), 2);

    let r1 = g.remove_edge(h0, h1);
    let r2 = g.remove_edge(h0, h1);
    // Should be able to remove the edge that we inserted.
    assert!(r1);
    // The edge should no longer be there!
    assert!(!r2);
}
