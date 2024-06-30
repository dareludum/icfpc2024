use std::{collections::HashMap, rc::Rc, usize};

use petgraph::{data::FromElements, graph::UnGraph, visit::DfsEvent};

use crate::{
    geometry::Vector2D,
    icfp::{serialize_str, Node, Value},
    runner::{Problem, Solution, Solver},
};

use crate::{
    icfp::{self, VarId},
    lambdaman_alt::LambdamanTreeWalk,
};

#[derive(Debug, Clone, Default)]
pub struct LambdamanTreeWalkLzCompressed {
    tree_walk: LambdamanTreeWalk,
}

impl Solver for LambdamanTreeWalkLzCompressed {
    fn name(&self) -> String {
        "lm:tree_walk_lz".to_owned()
    }

    fn initialize(&mut self, problem: Problem, _solution: Option<Solution>) {
        self.tree_walk.initialize(problem, _solution)
    }

    fn solve(&mut self) -> Solution {
        let (graph, player_pos_node_idx) = self.tree_walk.build_graph();

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

        let path = node_path
            .iter()
            .zip(node_path.iter().skip(1))
            .map(|(from, to)| {
                let from = tree[*from];
                let to = tree[*to];
                match to - from {
                    Vector2D { x: 1, y: 0 } => "R",
                    Vector2D { x: -1, y: 0 } => "L",
                    Vector2D { x: 0, y: 1 } => "D",
                    Vector2D { x: 0, y: -1 } => "U",
                    _ => unreachable!(),
                }
            })
            .collect::<String>();

        let plain_node = Rc::new(Node::Value(Value::Str(format!(
            "solve {} {}",
            self.tree_walk.problem.name, path
        ))));

        let mut best_len = serialize_str(plain_node.clone()).len();
        let mut best_ast = plain_node.clone();

        for l in (MIN_VARIABLE_DICT_ENTRY_COST + 1)..(best_len / 2) {
            let node = lz_compress_to_ast(
                format!("solve {} ", self.tree_walk.problem.name).as_str(),
                path.as_str(),
                l,
            );

            let compressed_len = serialize_str(node.clone()).len();
            let plain_len = serialize_str(plain_node.clone()).len();
            if compressed_len > plain_len {
                panic!(
                    "Compressed node (chunk size: {}) is bigger than plain node: {}/{}\n{:?}\n VS \n{:?}",
                    l, compressed_len, plain_len, node, plain_node
                );
            } else if compressed_len < plain_len {
                println!(
                    "Compressed node (chunk size: {}) is smaller than plain node: {}/{}",
                    l, compressed_len, plain_len
                );

                if compressed_len < best_len {
                    best_len = compressed_len;
                    best_ast = node;
                    println!("New best compression with chunk size: {}", l)
                }
            }
        }
        Solution::new(best_ast.clone(), serialize_str(best_ast).len() as u64)
    }
}

// Fixed cost: defining a lambda and calling it
// Example: S1 (len 2)
// Vs B$ La va S1 (len 11)
const MIN_FIXED_DICT_ENTRY_COST: usize = 9;

// Variable cost: concatenating a variable reference plus switching back to string nodes
// Example: B$ La va S1 (len 11) | B$ La B. va va S1 (len 17)
// Vs B$ La B. va va S1 (len 17) | B$ La B. va B. va S S1 (len 22)
const MIN_VARIABLE_DICT_ENTRY_COST: usize = 6 + 5;

fn saved_cost(pattern_len: usize, occurences: usize, dynamic_varlen_cost: usize) -> i64 {
    let plaintext_cost = (pattern_len * occurences) as i64;
    let dict_cost = (MIN_FIXED_DICT_ENTRY_COST
        + pattern_len
        + MIN_VARIABLE_DICT_ENTRY_COST * occurences
        + dynamic_varlen_cost * (occurences + 1)) as i64;
    plaintext_cost - dict_cost
}

enum LzChunk {
    Literal(String),
    Reference(u64),
}

// Basic lz compression, without ast variables reallocation
pub fn lz_compress_to_ast(msg_prefix: &str, path: &str, chunk_size: usize) -> icfp::NodeRef {
    let mut dict = HashMap::new();
    let mut dict_entries = Vec::new();
    let mut chunks_words = Vec::new();

    for i in (0..path.len()).step_by(chunk_size) {
        if i + chunk_size > path.len() {
            chunks_words.push(dict_entries.len());
            dict_entries.push(&path[i..]);
            break;
        }

        let chunk = &path[i..i + chunk_size];
        match dict.entry(chunk) {
            std::collections::hash_map::Entry::Occupied(entry) => {
                chunks_words.push(*entry.get());
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(dict_entries.len());
                chunks_words.push(dict_entries.len());
                dict_entries.push(chunk);
            }
        }
    }

    let hist = chunks_words
        .iter()
        .fold(HashMap::new(), |mut hist, &entry| {
            *hist.entry(entry).or_insert(0) += 1;
            hist
        });
    let max_varid_len = (hist.len() as f64).log(94.0).ceil() as usize;
    let do_substitution: HashMap<usize, bool> = hist
        .iter()
        .map(|(&entry, &occurences)| (entry, saved_cost(chunk_size, occurences, max_varid_len) > 0))
        .collect();

    // Maps a dict entry no to a variable ID
    let mut substitutions_vars = HashMap::new();
    let mut out_chunks = vec![LzChunk::Literal(msg_prefix.to_string())];

    for &entry in &chunks_words {
        if do_substitution[&entry] {
            let substitutions_no = substitutions_vars.len();
            let var_id = substitutions_vars.entry(entry).or_insert(substitutions_no);
            out_chunks.push(LzChunk::Reference(*var_id as u64));
        } else {
            if let Some(LzChunk::Literal(ref mut s)) = out_chunks.last_mut() {
                s.push_str(dict_entries[entry]);
            } else {
                out_chunks.push(LzChunk::Literal(dict_entries[entry].to_string()));
            }
        }
    }

    let vars: Vec<(usize, &str)> = substitutions_vars
        .iter()
        .map(|(&entry, &var_id)| (var_id, dict_entries[entry]))
        .collect();

    build_lz_chunks_ast(&vars, &out_chunks)
}

fn build_lz_chunks_ast(vars: &Vec<(usize, &str)>, chunks: &Vec<LzChunk>) -> icfp::NodeRef {
    let body_ast = build_lz_body_ast(&chunks);
    build_lz_lambdas_ast(vars, body_ast)
}

fn build_lz_body_ast(chunks: &Vec<LzChunk>) -> icfp::NodeRef {
    let mut ast = match chunks.first().unwrap() {
        LzChunk::Literal(s) => {
            icfp::NodeRef::new(icfp::Node::Value(icfp::Value::Str(s.to_string())))
        }
        LzChunk::Reference(var_id) => icfp::NodeRef::new(icfp::Node::Variable(VarId::new(*var_id))),
    };

    for chunk in chunks.iter().skip(1) {
        match chunk {
            LzChunk::Literal(s) => {
                if let icfp::Node::Value(_) = *ast {
                    unreachable!("cannot have two following literals");
                }
                ast = icfp::NodeRef::new(icfp::Node::BinaryOp {
                    op: icfp::BinaryOp::StrConcat,
                    left: ast,
                    right: icfp::NodeRef::new(icfp::Node::Value(icfp::Value::Str(s.to_string()))),
                });
            }
            LzChunk::Reference(var_id) => {
                ast = icfp::NodeRef::new(icfp::Node::BinaryOp {
                    op: icfp::BinaryOp::StrConcat,
                    left: ast,
                    right: icfp::NodeRef::new(icfp::Node::Variable(VarId::new(*var_id))),
                });
            }
        }
    }

    ast
}

fn build_lz_lambdas_ast(vars: &Vec<(usize, &str)>, body: icfp::NodeRef) -> icfp::NodeRef {
    let mut ast = body.clone();
    for (var_id, val) in vars {
        let lambda = icfp::NodeRef::new(icfp::Node::Lambda {
            var: VarId::new(*var_id as u64),
            body: ast,
        });

        let apply = icfp::NodeRef::new(icfp::Node::Apply {
            strat: icfp::EvalStrat::Name,
            f: lambda,
            value: icfp::NodeRef::new(icfp::Node::Value(icfp::Value::Str(val.to_string()))),
        });

        ast = apply;
    }

    ast
}
