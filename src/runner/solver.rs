use std::{collections::HashMap, rc::Rc};

use dyn_clone::DynClone;

use crate::icfp::{Node, Value};

use super::{Problem, Solution};

#[derive(Debug, Clone)]
pub enum Parameter {
    Int(i64),
    String(String),
}

pub trait Solver: DynClone + Sync + Send {
    fn name(&self) -> String;

    fn set_parameters(&mut self, parameters: HashMap<String, Parameter>) {
        assert!(
            parameters.is_empty(),
            "Solver {} doesn't accept parameters",
            self.name()
        );
    }
    fn initialize(&mut self, problem: Problem, solution: Option<Solution>);

    fn solve(&mut self) -> Solution;
}

dyn_clone::clone_trait_object!(Solver);

#[derive(Clone, Default)]
pub struct NoopSolver {
    problem: Problem,
}

impl Solver for NoopSolver {
    fn name(&self) -> String {
        "noop".to_owned()
    }

    fn initialize(&mut self, problem: Problem, _solution: Option<Solution>) {
        self.problem = problem;
    }

    fn solve(&mut self) -> Solution {
        Solution::new(Rc::new(Node::Value(Value::Str("".to_string()))), u64::MAX)
    }
}
