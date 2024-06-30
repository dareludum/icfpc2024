use std::{collections::HashMap, rc::Rc};

use petgraph::{
    data::FromElements,
    graph::{NodeIndex, UnGraph},
    visit::DfsEvent,
};

use crate::{
    geometry::Vector2D,
    icfp::{serialize_str, Node, Value},
    runner::{Problem, Solution, Solver},
};

use super::model::{Cell, LambdamanModel};

#[derive(Debug, Clone, Default)]
pub struct LambdamanTreeWalk {
    pub problem: Problem,
    pub model: LambdamanModel,
}

impl LambdamanTreeWalk {
    pub fn build_graph(&self) -> (UnGraph<Vector2D, ()>, NodeIndex) {
        let mut graph = UnGraph::default();

        let mut node_idx_map = HashMap::new();
        for (y, row) in self.model.map.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                if *cell == Cell::Wall {
                    continue;
                }
                let pos = Vector2D::new(x as i32, y as i32);
                node_idx_map.insert(pos, graph.add_node(pos));
            }
        }

        for (y, row) in self.model.map.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                if *cell == Cell::Wall {
                    continue;
                }
                let curr = Vector2D::new(x as i32, y as i32);
                if x > 0 && self.model.map[y][x - 1] != Cell::Wall {
                    let left = Vector2D::new(x as i32 - 1, y as i32);
                    graph.add_edge(node_idx_map[&curr], node_idx_map[&left], ());
                }
                if x + 1 < self.model.width && self.model.map[y][x + 1] != Cell::Wall {
                    let right = Vector2D::new(x as i32 + 1, y as i32);
                    graph.add_edge(node_idx_map[&curr], node_idx_map[&right], ());
                }
                if y > 0 && self.model.map[y - 1][x] != Cell::Wall {
                    let up = Vector2D::new(x as i32, y as i32 - 1);
                    graph.add_edge(node_idx_map[&curr], node_idx_map[&up], ());
                }
                if y + 1 < self.model.height && self.model.map[y + 1][x] != Cell::Wall {
                    let down = Vector2D::new(x as i32, y as i32 + 1);
                    graph.add_edge(node_idx_map[&curr], node_idx_map[&down], ());
                }
            }
        }

        (graph, node_idx_map[&self.model.player_pos])
    }
}

impl Solver for LambdamanTreeWalk {
    fn name(&self) -> String {
        "lm:tree_walk".to_owned()
    }

    fn initialize(&mut self, problem: Problem, _solution: Option<Solution>) {
        self.problem = problem;
        self.model = LambdamanModel::load(&self.problem.load());
    }

    fn solve(&mut self) -> Solution {
        let (graph, player_pos_node_idx) = self.build_graph();

        println!("Nodes count: {}", graph.node_count());
        println!("Edges count: {}", graph.edge_count());

        let min_spanning_tree = petgraph::algo::min_spanning_tree(&graph);
        let tree = UnGraph::<Vector2D, ()>::from_elements(min_spanning_tree);

        println!("Nodes count: {}", graph.node_count());
        println!("Edges count: {}", graph.edge_count());

        let mut node_path = vec![];
        petgraph::visit::depth_first_search(&tree, Some(player_pos_node_idx), |e| {
            if let DfsEvent::Discover(node, _) = e {
                if node_path.is_empty() {
                    node_path.push(node);
                } else {
                    let last = node_path.last().unwrap();
                    node_path.extend(
                        petgraph::algo::astar(&tree, *last, |f| f == node, |_| 1, |_| 0)
                            .unwrap()
                            .1
                            .into_iter()
                            .skip(1),
                    );
                }
            }
        });

        println!("Node path: {:?}", node_path);

        let path = node_path
            .iter()
            .zip(node_path.iter().skip(1))
            .map(|(from, to)| {
                let from = tree[*from];
                let to = tree[*to];
                println!("{:?} -> {:?}", from, to);
                match to - from {
                    Vector2D { x: 1, y: 0 } => "R",
                    Vector2D { x: -1, y: 0 } => "L",
                    Vector2D { x: 0, y: 1 } => "D",
                    Vector2D { x: 0, y: -1 } => "U",
                    _ => unreachable!(),
                }
            })
            .collect::<String>();

        println!("Path: {}", path);

        let node = Rc::new(Node::Value(Value::Str(format!(
            "solve {} {}",
            self.problem.name, path
        ))));
        Solution::new(node.clone(), serialize_str(node).len() as u64)
    }
}
