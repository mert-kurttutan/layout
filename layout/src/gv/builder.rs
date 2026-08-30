//! A graph builder that converts parsed AST trees to graphs.

use super::html::{parse_html_string, HtmlGrid};
use super::parser::ast::DotString;
use super::record::record_builder;
use crate::adt::dag::{NodeHandle, SubgraphHandle};
use crate::adt::map::ScopedMap;
use crate::core::base::Orientation;
use crate::core::color::Color;
use crate::core::style::*;
use crate::gv::parser::ast;
use crate::std_shapes::render::get_shape_size;
use crate::std_shapes::shapes::ShapeKind;
use crate::std_shapes::shapes::*;
use crate::topo::layout::VisualGraph;
use std::collections::HashMap;

type PropertyList = HashMap<String, DotString>;

// The methods in this file are responsible for converting the parsed Graphviz
// AST into the VisualGraph data-structure that we use for layout and rendering
// of the graph.

fn decode_quoted_label_entities(label: &str) -> String {
    let mut result = String::new();
    let mut chars = label.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '&' {
            result.push(ch);
            continue;
        }

        let mut entity = String::from("&");
        while let Some(&next) = chars.peek() {
            entity.push(next);
            chars.next();
            if next == ';' {
                break;
            }
        }

        if entity.ends_with(';') {
            if let Some(decoded) = decode_quoted_label_entity(&entity) {
                result.push(decoded);
            } else {
                result.push_str(&entity);
            }
        } else {
            result.push_str(&entity);
        }
    }

    result
}

fn decode_quoted_label_entity(entity: &str) -> Option<char> {
    let value = entity.strip_prefix("&#")?.strip_suffix(';')?;
    let codepoint = if let Some(hex) =
        value.strip_prefix('x').or_else(|| value.strip_prefix('X'))
    {
        u32::from_str_radix(hex, 16).ok()?
    } else {
        value.parse::<u32>().ok()?
    };
    char::from_u32(codepoint)
}

#[derive(Debug)]
struct EdgeDesc {
    from: String,
    to: String,
    props: PropertyList,
    is_directed: bool,
    from_port: Option<String>,
    to_port: Option<String>,
}

#[derive(Debug)]
struct SubgraphDesc {
    name: String,
    parent: usize,
    props: PropertyList,
}

/// This class constructs a visual graph from the parsed AST.
#[derive(Debug)]
pub struct GraphBuilder {
    // This records the state of the top-level graph.
    global_state: PropertyList,
    subgraphs: Vec<SubgraphDesc>,
    subgraph_stack: Vec<usize>,
    node_subgraphs: HashMap<String, usize>,
    // This keeps track of the construction order of the nodes, because
    // hashmap does not maintain a persistent iteration order.
    node_order: Vec<String>,
    // Maps node names to their property list.
    nodes: HashMap<String, PropertyList>,
    // A list of edge properties.
    edges: Vec<EdgeDesc>,
    /// Scopes that maintain the property list that changes as we enter and
    /// leave different regions of the graph.
    global_attr: ScopedMap<String, DotString>,
    node_attr: ScopedMap<String, DotString>,
    edge_attr: ScopedMap<String, DotString>,
}
impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            global_state: PropertyList::new(),
            subgraphs: vec![SubgraphDesc {
                name: String::from("main"),
                parent: 0,
                props: PropertyList::new(),
            }],
            subgraph_stack: vec![0],
            node_subgraphs: HashMap::new(),
            node_order: Vec::new(),
            nodes: HashMap::new(),
            edges: Vec::new(),
            global_attr: ScopedMap::new(),
            node_attr: ScopedMap::new(),
            edge_attr: ScopedMap::new(),
        }
    }
    pub fn visit_graph(&mut self, graph: &ast::Graph) {
        self.global_attr.push();
        self.node_attr.push();
        self.edge_attr.push();
        self.subgraphs[0].name = graph.name.clone();
        for stmt in &graph.list.list {
            self.visit_stmt(stmt);
        }

        // TODO: we dump the property list when we close the scope. This is not
        // correct for sub graphs.
        self.global_state = self.global_attr.flatten();
        self.subgraphs[0].props = self.global_state.clone();

        self.global_attr.pop();
        self.node_attr.pop();
        self.edge_attr.pop();
    }
    fn visit_stmt(&mut self, stmt: &ast::Stmt) {
        match stmt {
            ast::Stmt::Edge(e) => {
                self.visit_edge(e);
            }
            ast::Stmt::Node(n) => {
                self.visit_node(n);
            }
            ast::Stmt::Attribute(a) => {
                self.visit_att(a);
            }
            ast::Stmt::SubGraph(g) => {
                self.visit_subgraph(g);
            }
        }
    }

    fn visit_subgraph(&mut self, graph: &ast::Graph) {
        let parent = *self.subgraph_stack.last().unwrap_or(&0);
        let is_cluster = graph.name.starts_with("cluster_");
        let subgraph_idx = if is_cluster {
            let idx = self.subgraphs.len();
            self.subgraphs.push(SubgraphDesc {
                name: graph.name.clone(),
                parent,
                props: PropertyList::new(),
            });
            self.subgraph_stack.push(idx);
            idx
        } else {
            parent
        };

        self.global_attr.push();
        self.node_attr.push();
        self.edge_attr.push();

        for stmt in &graph.list.list {
            self.visit_stmt(stmt);
        }

        if is_cluster {
            self.subgraphs[subgraph_idx].props = self.global_attr.flatten();
        }

        self.global_attr.pop();
        self.node_attr.pop();
        self.edge_attr.pop();

        if is_cluster {
            self.subgraph_stack.pop();
        }
    }

    fn visit_edge(&mut self, e: &ast::EdgeStmt) {
        self.edge_attr.push();

        for att in e.list.iter() {
            self.edge_attr.insert(&att.0, &att.1);
        }

        self.init_node_with_name(&e.from.name, false);

        let mut prev = &e.from.name;
        for dest in &e.to {
            let curr = &dest.0.name;
            self.init_node_with_name(curr, false);

            let has_arrow = matches!(dest.1, ast::ArrowKind::Arrow);
            let prop_list = self.edge_attr.flatten();

            let edge = EdgeDesc {
                from: prev.clone(),
                to: curr.clone(),
                props: prop_list,
                is_directed: has_arrow,
                from_port: e.from.port.clone(),
                to_port: dest.0.port.clone(),
            };
            self.edges.push(edge);
            prev = curr;
        }
        self.edge_attr.pop();
    }

    // If \p overwrite is set then we are declaring a node. This means that
    // we need to update the properties that already exist.
    fn init_node_with_name(&mut self, name: &str, overwrite: bool) {
        let node_attr = self.node_attr.flatten();

        if let Option::Some(prop_list) = self.nodes.get_mut(name) {
            if !overwrite {
                return;
            }
            for p in node_attr {
                prop_list.insert(p.0, p.1);
            }
        } else {
            self.node_order.push(name.to_string());
            self.node_subgraphs.insert(
                name.to_string(),
                *self.subgraph_stack.last().unwrap_or(&0),
            );
            self.nodes.insert(name.to_string(), node_attr);
        }
    }

    fn visit_node(&mut self, n: &ast::NodeStmt) {
        self.node_attr.push();

        for att in n.list.iter() {
            self.node_attr.insert(&att.0, &att.1);
        }

        self.init_node_with_name(&n.id.name, true);
        self.node_attr.pop();
    }

    fn visit_att(&mut self, att: &ast::AttrStmt) {
        match att.target {
            ast::AttrStmtTarget::Graph => {
                for att in att.list.iter() {
                    self.global_attr.insert(&att.0, &att.1);
                }
            }
            ast::AttrStmtTarget::Node => {
                for att in att.list.iter() {
                    self.node_attr.insert(&att.0, &att.1);
                }
            }
            ast::AttrStmtTarget::Edge => {
                for att in att.list.iter() {
                    self.edge_attr.insert(&att.0, &att.1);
                }
            }
        }
    }

    pub fn get(&self) -> VisualGraph {
        self.try_get()
            .expect("failed to build visual graph from DOT AST")
    }

    pub fn try_get(&self) -> Result<VisualGraph, String> {
        let mut dir = Orientation::TopToBottom;

        // Set the graph orientation based on the 'rankdir' property.
        if let Option::Some(DotString::String(rd)) =
            self.global_state.get("rankdir")
        {
            if rd == "LR" {
                dir = Orientation::LeftToRight;
            }
        }

        let mut vg = VisualGraph::new(dir);
        if let Some(label) =
            Self::get_graph_label_from_attributes(dir, &self.global_state)?
        {
            vg.set_graph_label(label);
        }

        let mut subgraph_handles = vec![SubgraphHandle::new(0)];
        for subgraph in self.subgraphs.iter().skip(1) {
            let parent = subgraph_handles[subgraph.parent];
            let elem = Self::get_subgraph_shape_from_attributes(
                dir,
                &subgraph.props,
                &subgraph.name,
            )?;
            let handle = vg.add_subgraph(elem, parent);
            subgraph_handles.push(handle);
        }

        // Keeps track of the newly created nodes and indexes them by name.
        let mut node_map: HashMap<String, NodeHandle> = HashMap::new();

        assert_eq!(self.nodes.len(), self.node_order.len());

        // Create and register all of the nodes.
        for node_name in self.node_order.iter() {
            let node_prop = self.nodes.get(node_name).expect(
                "this is a bug: node_name should have been recorded in the node property map; please open a GitHub issue",
            );

            let shape =
                Self::get_shape_from_attributes(dir, node_prop, node_name)?;
            let subgraph_idx =
                *self.node_subgraphs.get(node_name).unwrap_or(&0);
            let handle =
                vg.add_node_to_subgraph(shape, subgraph_handles[subgraph_idx]);
            node_map.insert(node_name.to_string(), handle);
        }

        // Create and register all of the edges.
        for edge_prop in &self.edges {
            let shape = Self::get_arrow_from_attributes(
                &edge_prop.props,
                edge_prop.is_directed,
                edge_prop.from_port.clone(),
                edge_prop.to_port.clone(),
            )?;
            let from = node_map.get(&edge_prop.from).expect(
                "this is a bug: edge source should have been recorded in the node handle map; please open a GitHub issue",
            );
            let to = node_map.get(&edge_prop.to).expect(
                "this is a bug: edge target should have been recorded in the node handle map; please open a GitHub issue",
            );
            vg.add_edge(shape, *from, *to);
        }

        Ok(vg)
    }

    fn get_arrow_from_attributes(
        lst: &PropertyList,
        has_arrow: bool,
        from_port: Option<String>,
        to_port: Option<String>,
    ) -> Result<Arrow, String> {
        let mut line_width = 1;
        let mut font_size: usize = 14;
        let start = LineEndKind::None;
        let end = if has_arrow {
            LineEndKind::Arrow
        } else {
            LineEndKind::None
        };
        let mut label = Option::None;
        let mut head_label = Option::None;
        let mut tail_label = Option::None;
        let mut color = String::from("black");
        let mut line_style = LineStyleKind::Normal;

        if let Option::Some(val) = lst.get(&"label".to_string()) {
            label = Self::get_label_content(val)?;
        }

        if let Option::Some(val) = lst.get(&"headlabel".to_string()) {
            head_label = Self::get_label_content(val)?;
        }

        if let Option::Some(val) = lst.get(&"taillabel".to_string()) {
            tail_label = Self::get_label_content(val)?;
        }

        if let Option::Some(DotString::String(stl)) =
            lst.get(&"style".to_string())
        {
            if stl == "dashed" {
                line_style = LineStyleKind::Dashed;
            }
        }

        if let Option::Some(DotString::String(x)) =
            lst.get(&"color".to_string())
        {
            color = x.clone();
            color = Self::normalize_color(color);
        }

        if let Option::Some(DotString::String(pw)) =
            lst.get(&"penwidth".to_string())
        {
            if let Result::Ok(x) = pw.parse::<usize>() {
                line_width = x;
            } else {
                #[cfg(feature = "log")]
                log::info!("Can't parse integer \"{}\"", pw);
            }
        }

        if let Option::Some(DotString::String(fx)) =
            lst.get(&"fontsize".to_string())
        {
            if let Result::Ok(x) = fx.parse::<usize>() {
                font_size = x;
            } else {
                #[cfg(feature = "log")]
                log::info!("Can't parse integer \"{}\"", fx);
            }
        }

        let color = Color::fast(&color);
        let look = StyleAttr::new(color, line_width, None, 0, font_size);
        let mut arrow = Arrow::new_with_content(
            start, end, line_style, label, &look, &from_port, &to_port,
        );
        arrow.head_label = head_label;
        arrow.tail_label = tail_label;
        Ok(arrow)
    }

    fn get_label_content(
        val: &DotString,
    ) -> Result<Option<ShapeContent>, String> {
        match val {
            DotString::String(val) => {
                if val.is_empty() {
                    Ok(Option::None)
                } else {
                    Ok(Option::Some(ShapeContent::String(
                        decode_quoted_label_entities(val),
                    )))
                }
            }
            DotString::HtmlString(val) => {
                parse_html_string(val).map(ShapeContent::Html).map(Some)
            }
        }
    }

    /// Convert the color to some color that we can handle.
    fn normalize_color(color: String) -> String {
        let mut color = color;
        if let Option::Some(idx) = color.find(':') {
            color = color[0..idx].to_string();
        }
        if color == "transparent" {
            color = "white".to_string();
        }
        color
    }

    fn get_shape_from_attributes(
        dir: Orientation,
        lst: &PropertyList,
        default_name: &str,
    ) -> Result<Element, String> {
        let mut label = ShapeContent::String(default_name.to_string());
        let mut edge_color = String::from("black");
        let mut fill_color = String::from("white");
        let mut font_size: usize = 14;
        let mut line_width: usize = 1;
        let mut make_xy_same = false;
        let mut rounded_corder_value = 0;
        // let mut shape = ShapeKind::Circle(label.clone());
        let mut shape = ShapeKind::Circle(label.clone());

        if let Option::Some(x) = lst.get(&"label".to_string()) {
            // label = val.clone();
            match x {
                DotString::String(val) => {
                    let decoded_label = decode_quoted_label_entities(val);
                    label = ShapeContent::String(decoded_label.clone());
                    shape =
                        ShapeKind::Circle(ShapeContent::String(decoded_label));
                }
                DotString::HtmlString(val) => {
                    label = ShapeContent::Html(parse_html_string(val)?);
                    shape = ShapeKind::None(label.clone());
                }
            }
        }

        // Set the shape.
        if let Option::Some(DotString::String(val)) =
            lst.get(&"shape".to_string())
        {
            match &val[..] {
                "box" => {
                    shape = ShapeKind::Box(label);
                    make_xy_same = false;
                }
                "doublecircle" => {
                    shape = ShapeKind::DoubleCircle(label);
                    make_xy_same = true;
                }
                "record" => {
                    // shape = record_builder(&label);
                    match label {
                        ShapeContent::String(s) => {
                            shape = record_builder(&s);
                        }
                        ShapeContent::Html(_) => {}
                    }
                }
                "Mrecord" => {
                    rounded_corder_value = 15;
                    match label {
                        ShapeContent::String(s) => {
                            shape = record_builder(&s);
                        }
                        ShapeContent::Html(_) => {}
                    }
                }
                "circle" => {
                    shape = ShapeKind::Circle(label);
                    make_xy_same = true;
                }
                _ => {}
            }
        }

        if let Option::Some(DotString::String(x)) =
            lst.get(&"color".to_string())
        {
            edge_color = x.clone();
            edge_color = Self::normalize_color(edge_color);
        }

        if let Option::Some(DotString::String(style)) =
            lst.get(&"style".to_string())
        {
            if style == "filled" && !lst.contains_key("fillcolor") {
                fill_color = "lightgray".to_string();
            }
        }

        if let Option::Some(DotString::String(x)) =
            lst.get(&"fillcolor".to_string())
        {
            fill_color = x.clone();
            fill_color = Self::normalize_color(fill_color);
        }

        if let Option::Some(DotString::String(fx)) =
            lst.get(&"fontsize".to_string())
        {
            if let Result::Ok(x) = fx.parse::<usize>() {
                font_size = x;
            } else {
                #[cfg(feature = "log")]
                log::info!("Can't parse integer \"{}\"", fx);
            }
        }

        if let Option::Some(DotString::String(pw)) =
            lst.get(&"width".to_string())
        {
            if let Result::Ok(x) = pw.parse::<usize>() {
                line_width = x;
            } else {
                #[cfg(feature = "log")]
                log::info!("Can't parse integer \"{}\"", pw);
            }
        }

        // We flip the orientation before we create the shape. In graphs that
        // grow top down the records grow to the left.
        let dir = dir.flip();

        match &mut shape {
            ShapeKind::Circle(ShapeContent::Html(HtmlGrid::FontTable(x))) => {
                x.resize(font_size);
            }
            ShapeKind::Box(ShapeContent::Html(HtmlGrid::FontTable(x))) => {
                x.resize(font_size);
            }
            ShapeKind::DoubleCircle(ShapeContent::Html(
                HtmlGrid::FontTable(x),
            )) => {
                x.resize(font_size);
            }
            ShapeKind::None(ShapeContent::Html(HtmlGrid::FontTable(x))) => {
                x.resize(font_size);
            }
            _ => {}
        }

        let sz = get_shape_size(dir, &shape, font_size, make_xy_same);
        let look = StyleAttr::new(
            Color::fast(&edge_color),
            line_width,
            Option::Some(Color::fast(&fill_color)),
            rounded_corder_value,
            font_size,
        );
        Ok(Element::create(shape, look, dir, sz))
    }

    fn get_graph_label_from_attributes(
        dir: Orientation,
        lst: &PropertyList,
    ) -> Result<Option<Element>, String> {
        let Some(mut label) = lst
            .get("label")
            .map(Self::get_label_content)
            .transpose()?
            .flatten()
        else {
            return Ok(None);
        };
        let mut font_size: usize = 14;
        let valign = Self::get_label_location(lst, VAlign::Top);

        if let Option::Some(DotString::String(fx)) =
            lst.get(&"fontsize".to_string())
        {
            if let Result::Ok(x) = fx.parse::<usize>() {
                font_size = x;
            }
        }

        if let ShapeContent::Html(HtmlGrid::FontTable(table)) = &mut label {
            table.resize(font_size);
        }

        let shape = ShapeKind::None(label);
        let mut look =
            StyleAttr::new(Color::fast("black"), 0, None, 0, font_size);
        look.valign = valign;
        let sz = get_shape_size(dir, &shape, font_size, false);
        Ok(Some(Element::create(shape, look, dir, sz)))
    }

    fn get_label_location(lst: &PropertyList, default: VAlign) -> VAlign {
        match lst.get("labelloc") {
            Some(DotString::String(value)) if value == "b" => VAlign::Bottom,
            Some(DotString::String(value)) if value == "t" => VAlign::Top,
            _ => default,
        }
    }

    fn get_subgraph_shape_from_attributes(
        dir: Orientation,
        lst: &PropertyList,
        default_name: &str,
    ) -> Result<Element, String> {
        let label = lst
            .get("label")
            .map(Self::get_label_content)
            .transpose()?
            .flatten()
            .or_else(|| {
                if default_name.is_empty() || default_name == "main" {
                    None
                } else {
                    Some(ShapeContent::String(default_name.to_string()))
                }
            });
        let mut edge_color = String::from("black");
        let mut fill_color: Option<String> = None;
        let mut font_size: usize = 14;
        let line_width: usize = 1;
        let valign = Self::get_label_location(lst, VAlign::Top);

        if let Option::Some(DotString::String(x)) =
            lst.get(&"color".to_string())
        {
            edge_color = Self::normalize_color(x.clone());
        }

        if let Option::Some(DotString::String(style)) =
            lst.get(&"style".to_string())
        {
            if style == "filled" && !lst.contains_key("fillcolor") {
                fill_color = Some(edge_color.clone());
            }
        }

        if let Option::Some(DotString::String(x)) =
            lst.get(&"fillcolor".to_string())
        {
            fill_color = if x == "transparent" {
                None
            } else {
                Some(Self::normalize_color(x.clone()))
            };
        }

        if let Option::Some(DotString::String(fx)) =
            lst.get(&"fontsize".to_string())
        {
            if let Result::Ok(x) = fx.parse::<usize>() {
                font_size = x;
            }
        }

        let mut look = StyleAttr::new(
            Color::fast(&edge_color),
            line_width,
            fill_color.map(|color| Color::fast(&color)),
            0,
            font_size,
        );
        look.valign = valign;
        Ok(Element::create_subgraph(dir, label, &look))
    }
}
