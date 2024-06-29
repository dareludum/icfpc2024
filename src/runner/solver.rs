use std::collections::HashMap;

use dyn_clone::DynClone;

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
