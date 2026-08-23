//! This module the implementation of VisualGraph, which is the data-structure
//! that we use for assigning (x,y) locations to all of the shapes and edges.
//! The VisualGraph uses a DAG to represent the relationships between the nodes
//! and the Ranks data-structure to represent rows of shapes that have the same
//! x coordinate.

#[cfg(feature = "log")]
extern crate log;

use crate::adt::dag::*;
use crate::core::base::Orientation;
use crate::core::format::RenderBackend;
use crate::core::format::Renderable;
use crate::core::format::Visible;
use crate::core::geometry::get_size_for_str;
use crate::core::geometry::Point;
use crate::core::geometry::Position;
use crate::core::style::StyleAttr;
use crate::std_shapes::render::*;
use crate::std_shapes::shapes::*;
use crate::topo::optimizer::EdgeCrossOptimizer;
use crate::topo::optimizer::RankOptimizer;
use crate::topo::sander;
use std::mem::swap;
use std::vec;

use super::placer::Placer;

#[derive(Debug)]
pub struct VisualGraph {
    // Optional top-level graph label.
    graph_label: Option<Element>,
    // Holds all of the elements in the graph.
    nodes: Vec<Element>,
    // Holds all of the subgraph frames in the graph.
    subgraphs: Vec<Element>,
    // The arrows and the list of elements that they visits.
    edges: Vec<(Arrow, Vec<NodeHandle>)>,
    // Contains a list of self-edges. We use this as a temporary storage during
    // lowering. This list should be removes by the time we start the layout
    // process.
    self_edges: Vec<(Arrow, NodeHandle)>,
    // Representing the connections between the nodes. Used to keep the graph
    // a dag by detecting reverse edges. Used to create 'levels', and decide
    // which node moves/controls which node. After lowering, the graph should
    // only contain edges that skip zero or one levels.
    pub dag: DAG,
    // Sets the graph orientation (L-to-R, or T-to-B).
    orientation: Orientation,
}

impl VisualGraph {
    pub fn new(orientation: Orientation) -> Self {
        let main_graph = Element::create_subgraph(
            orientation,
            None,
            &StyleAttr::new(
                crate::core::color::Color::transparent(),
                0,
                None,
                0,
                14,
            ),
        );
        VisualGraph {
            graph_label: None,
            nodes: Vec::new(),
            subgraphs: vec![main_graph],
            edges: Vec::new(),
            self_edges: Vec::new(),
            dag: DAG::new(),
            orientation,
        }
    }

    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    pub fn num_nodes(&self) -> usize {
        self.dag.len()
    }

    pub fn set_graph_label(&mut self, elem: Element) {
        self.graph_label = Some(elem);
    }

    pub fn iter_nodes(&self) -> NodeIterator {
        self.dag.iter()
    }

    pub fn succ(&self, node: NodeHandle) -> &Vec<NodeHandle> {
        self.dag.successors(node)
    }

    pub fn preds(&self, node: NodeHandle) -> &Vec<NodeHandle> {
        self.dag.predecessors(node)
    }

    pub fn pos(&self, n: NodeHandle) -> Position {
        self.element(n).position()
    }

    pub fn pos_mut(&mut self, n: NodeHandle) -> &mut Position {
        self.element_mut(n).position_mut()
    }

    pub fn pos_sg(&self, sg: SubgraphHandle) -> Position {
        self.subgraphs[sg.get_index()].position()
    }

    pub fn pos_sg_mut(&mut self, sg: SubgraphHandle) -> &mut Position {
        self.subgraphs[sg.get_index()].position_mut()
    }

    pub fn size_sg_label(&self, sg: SubgraphHandle) -> Point {
        let font_size = self.subgraphs[sg.get_index()].look.font_size;
        match &self.subgraphs[sg.get_index()].shape {
            ShapeKind::Frame(Some(ShapeContent::String(label))) => {
                get_size_for_str(label, font_size)
            }
            ShapeKind::Frame(Some(ShapeContent::Html(html))) => {
                html.size(font_size)
            }
            ShapeKind::Frame(None) => Point::zero(),
            _ => panic!("Subgraph does not have a frame shape"),
        }
    }

    pub fn is_connector(&self, n: NodeHandle) -> bool {
        return self.element(n).is_connector();
    }

    pub fn transpose(&mut self) {
        if let Some(label) = &mut self.graph_label {
            label.transpose();
        }
        for node in self.dag.iter() {
            self.element_mut(node).transpose();
        }
        for subgraph in &mut self.subgraphs {
            subgraph.transpose();
        }
    }

    pub fn element(&self, node: NodeHandle) -> &Element {
        &self.nodes[node.get_index()]
    }

    pub fn element_mut(&mut self, node: NodeHandle) -> &mut Element {
        &mut self.nodes[node.get_index()]
    }

    /// Add a node to the graph.
    /// \returns a handle to the node.
    pub fn add_node(&mut self, elem: Element) -> NodeHandle {
        self.add_node_to_subgraph(elem, SubgraphHandle::new(0))
    }

    pub fn add_node_to_subgraph(
        &mut self,
        elem: Element,
        parent_subgraph_idx: SubgraphHandle,
    ) -> NodeHandle {
        let res = self.dag.new_node(parent_subgraph_idx);
        assert!(res.get_index() == self.nodes.len());
        self.nodes.push(elem);
        res
    }

    pub fn add_connector_node(
        &mut self,
        elem: Element,
        from: NodeHandle,
        to: NodeHandle,
    ) -> NodeHandle {
        let res = self.dag.new_connector_node(from, to);
        assert!(res.get_index() == self.nodes.len());
        self.nodes.push(elem);
        res
    }

    pub(crate) fn add_vertical_border(&mut self, lvl_s: usize, lvl_e: usize) {
        (lvl_s..=lvl_e).for_each(|_| {
            self.nodes.push(Element::create_border());
            self.nodes.push(Element::create_border());
        });
    }

    /// Add an edge to the graph.
    pub fn add_edge(&mut self, arrow: Arrow, from: NodeHandle, to: NodeHandle) {
        assert!(from.get_index() < self.nodes.len(), "Invalid handle");
        assert!(to.get_index() < self.nodes.len(), "Invalid handle");
        let lst = vec![from, to];
        self.edges.push((arrow, lst));
    }

    pub fn add_subgraph(
        &mut self,
        elem: Element,
        parent_subgraph_idx: SubgraphHandle,
    ) -> SubgraphHandle {
        let res = self.dag.new_subgraph(parent_subgraph_idx);
        assert!(res.get_index() == self.subgraphs.len());
        self.subgraphs.push(elem);
        res
    }
}

// Render.
impl VisualGraph {
    fn render(&self, debug: bool, rb: &mut dyn RenderBackend) {
        if let Some(label) = &self.graph_label {
            label.render(debug, rb);
        }
        for subgraph in &self.subgraphs {
            subgraph.render(debug, rb);
        }
        // Draw the nodes.
        for node in &self.nodes {
            node.render(debug, rb);
        }

        // Draw the arrows:
        for arrow in &self.edges {
            let mut elements = Vec::new();
            for h in &arrow.1 {
                elements.push(self.nodes[h.get_index()].clone());
            }
            render_arrow(rb, debug, &elements[..], &arrow.0);
        }
    }
}

impl VisualGraph {
    pub fn do_it(
        &mut self,
        debug_mode: bool,
        disable_opt: bool,
        disable_layout: bool,
        rb: &mut dyn RenderBackend,
    ) {
        if self.dag.is_empty() {
            return;
        }
        self.lower(disable_opt);
        sander::do_it(self);
        Placer::new(self).layout(disable_layout);
        self.place_graph_label();
        self.render(debug_mode, rb);
    }

    fn lower(&mut self, disable_optimizations: bool) {
        #[cfg(feature = "log")]
        log::info!("Lowering a graph with {} nodes.", self.num_nodes());
        self.to_valid_dag();
        self.split_text_edges();
        self.split_long_edges(disable_optimizations);

        for elem in self.dag.iter() {
            self.element_mut(elem).resize();
        }
    }

    fn place_graph_label(&mut self) {
        let Some(label) = &mut self.graph_label else {
            return;
        };

        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        for node in &self.nodes {
            let bbox = node.position().bbox(false);
            min_x = min_x.min(bbox.0.x);
            max_x = max_x.max(bbox.1.x);
            min_y = min_y.min(bbox.0.y);
        }
        for subgraph in &self.subgraphs {
            let bbox = subgraph.position().bbox(false);
            min_x = min_x.min(bbox.0.x);
            max_x = max_x.max(bbox.1.x);
            min_y = min_y.min(bbox.0.y);
        }
        if !min_x.is_finite() || !max_x.is_finite() || !min_y.is_finite() {
            return;
        }

        let label_height = label.position().size(false).y;
        let margin = 10.;
        let label_center = Point::new(
            (min_x + max_x) / 2.,
            min_y - margin - label_height / 2.,
        );
        label.position_mut().move_to(label_center);

        let shift_y = (label_height + margin * 2.).max(0.);
        label.position_mut().translate(Point::new(0., shift_y));
        for node in &mut self.nodes {
            node.position_mut().translate(Point::new(0., shift_y));
        }
        for subgraph in &mut self.subgraphs {
            subgraph.position_mut().translate(Point::new(0., shift_y));
        }

        let label_min_x = label.position().bbox(false).0.x;
        if label_min_x < 0. {
            let shift_x = -label_min_x + margin;
            label.position_mut().translate(Point::new(shift_x, 0.));
            for node in &mut self.nodes {
                node.position_mut().translate(Point::new(shift_x, 0.));
            }
            for subgraph in &mut self.subgraphs {
                subgraph.position_mut().translate(Point::new(shift_x, 0.));
            }
        }
    }

    /// Flip the edges in the graph to create a valid dag.
    /// This is the first step of graph canonicalization.
    pub fn to_valid_dag(&mut self) {
        let edges = self.edges.clone();
        self.edges.clear();

        // At this point the DAG should have all of the nodes, but none of the
        // edges. In here we construct the edges.
        assert_eq!(self.nodes.len(), self.dag.len(), "bad number of nodes");

        // For each edge.
        for edge in edges {
            let mut arrow = edge.0;
            let lst = edge.1;
            assert_eq!(lst.len(), 2);
            let mut from = lst[0];
            let mut to = lst[1];

            if from == to {
                self.self_edges.push((arrow, from));
                continue;
            }

            // Reverse back edges.
            if self.dag.is_reachable(to, from) {
                swap(&mut from, &mut to);
                arrow = arrow.reverse();
            }

            self.dag.add_edge(from, to);
            self.add_edge(arrow, from, to);

            self.dag.verify();
        }
    }

    /// Convert all of the edges that contain text labels to edges that go
    /// through connectors.
    /// This is the second step of graph canonicalization.
    pub fn split_text_edges(&mut self) {
        let mut edges = self.edges.clone();
        //self.edge_list.clear();

        for edge in edges.iter_mut() {
            let lst = &edge.1;
            assert_eq!(lst.len(), 2);
            let arrow = &edge.0;
            let from = lst[0];
            let to = lst[1];

            // If the edge is empty then there is nothing to do.
            if edge.0.text.is_none() {
                continue;
            }

            let text = arrow.text.clone().unwrap();

            // Create a new connection block.
            let dir = self.element(from).orientation;
            let conn =
                Element::create_connector_with_content(text, &arrow.look, dir);
            let conn = self.add_connector_node(conn, from, to);

            // Update the edge node list, and remove the text.
            edge.1 = vec![from, conn, to];
            edge.0.text = Option::None;

            // Add the edge to dag.
            let res = self.dag.remove_edge(from, to);
            assert!(res, "Expected the edge to be in the graph!");
            self.dag.add_edge(from, conn);
            self.dag.add_edge(conn, to);
        }

        self.edges = edges;
    }

    pub fn split_long_edges(&mut self, disable_optimizations: bool) {
        // Assign optimal rank to nodes in the graph.
        self.dag.recompute_node_ranks();
        self.dag.verify();
        if !disable_optimizations {
            RankOptimizer::new(&mut self.dag).optimize();
        }

        let mut edges = self.edges.clone();
        self.edges.clear();

        for edge in edges.iter_mut() {
            let mut lst = edge.1.clone();

            // Points the 'to' edge in each pair in the graph. We start with
            // node '1', and compare to the previous node.
            let mut i = 1;
            while i < lst.len() {
                let prev = lst[i - 1];
                let curr = lst[i];

                let prev_level = self.dag.level(prev);
                let curr_level = self.dag.level(curr);

                // If the edge already spans zero or one rank, there is no
                // long edge to split.
                if prev_level + 1 >= curr_level {
                    i += 1;
                    continue;
                }

                // We need to add a new connector node.
                let dir = self.element(prev).orientation;
                let conn = Element::empty_connector(dir);
                let conn = self.add_connector_node(conn, prev, curr);
                lst.insert(i, conn);

                // Update the dag connections.
                self.dag.remove_edge(prev, curr);
                self.dag.add_edge(prev, conn);
                self.dag.add_edge(conn, curr);

                // Place the new connection node at the right level.
                self.dag.update_node_rank_level(conn, prev_level + 1, None);
            }

            edge.1 = lst;
        }
        self.edges = edges;

        if !disable_optimizations {
            EdgeCrossOptimizer::new(&mut self.dag).optimize();
        }
        self.expand_self_edges()
    }

    /// Convert all of the saved self edges into proper edges in the graph.
    pub fn expand_self_edges(&mut self) {
        for se in self.self_edges.clone().iter() {
            let mut arrow = se.0.clone();
            let node = se.1;
            let level = self.dag.level(node);
            let text = arrow.text.clone();
            arrow.text = Option::None;
            let dir = self.element(node).orientation;
            let conn = if let Some(text) = text {
                Element::create_connector_with_content(text, &arrow.look, dir)
            } else {
                Element::create_connector("", &arrow.look, dir)
            };
            let conn = self.add_node_to_subgraph(
                conn,
                self.dag.get_parent_subgraph_index_n(node),
            );
            self.dag.update_node_rank_level(conn, level, Some(node));
            self.edges.push((arrow, vec![node, conn, node]));
        }

        // Wipe out the self edges.
        self.self_edges.clear();
    }
}
