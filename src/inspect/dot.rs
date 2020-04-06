use crate::inspect::*;

type Id = usize;

#[derive(Copy, Clone, Debug)]
enum Shape {
    Box,
    Circle,
    Diamond,
    Ellipse,
    Oval,
    Triangle,
}

impl Shape {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Box => "box",
            Self::Circle => "circle",
            Self::Diamond => "diamond",
            Self::Ellipse => "ellipse",
            Self::Oval => "oval",
            Self::Triangle => "triangle",
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum Color {
    Black,
    Green,
    Orange,
    Red,
    Yellow,
}

impl Color {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Black => "black",
            Self::Green => "green",
            Self::Orange => "orange",
            Self::Red => "red",
            Self::Yellow => "yellow",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    id: Id,
    label: String,
    shape: Shape,
    color: Color,
}

#[derive(Copy, Clone, Debug)]
pub struct Edge((Id, Id));

#[derive(Clone, Debug)]
pub struct Graph {
    next_id: Id,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl Graph {
    fn next_id(&mut self) -> Id {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }
}

use std::fmt;

impl fmt::Display for Graph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "digraph tree {{")?;

        for node in &self.nodes {
            writeln!(
                f,
                r#"    N{}[label="{}",shape={},color={}];"#,
                node.id,
                node.label,
                node.shape.as_str(),
                node.color.as_str()
            )?;
        }

        for Edge((source, target)) in &self.edges {
            writeln!(f, r#"    N{} -> N{};"#, source, target,)?;
        }

        writeln!(f, "}}")
    }
}

fn eval_color(eval: bool) -> Color {
    if eval {
        Color::Green
    } else {
        Color::Red
    }
}

fn tree_to_graph(tree: &PredTree, graph: &mut Graph) -> Id {
    match tree {
        Tree::Leaf(Label((label, eval))) => {
            let id = graph.next_id();

            graph.add_node(Node {
                id,
                label: label.to_string(),
                shape: Shape::Box,
                color: eval_color(*eval),
            });

            id
        }
        Tree::Node { label, children } => {
            let id = graph.next_id();
            let Label((label, eval)) = label;

            let shape = if children.len() <= 1 {
                Shape::Oval
            } else {
                Shape::Triangle
            };

            graph.add_node(Node {
                id,
                label: label.to_string(),
                shape,
                color: eval_color(*eval),
            });

            for child in children {
                let child_id = tree_to_graph(child, graph);
                graph.add_edge(Edge((id, child_id)));
            }

            id
        }
    }
}

impl PredTree {
    pub fn to_graph(&self) -> Graph {
        let mut graph = Graph {
            next_id: 0,
            nodes: vec![],
            edges: vec![],
        };

        tree_to_graph(self, &mut graph);
        graph
    }
}
