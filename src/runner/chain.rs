use super::{Problem, Solution, Solver};

#[derive(Clone)]
pub struct Chain {
    solver0: Box<dyn Solver>,
    solver1: Box<dyn Solver>,
    problem: Problem,
}

impl Solver for Chain {
    fn name(&self) -> String {
        format!("{}+{}", self.solver0.name(), self.solver1.name())
    }

    fn initialize(&mut self, problem: Problem, solution: Option<Solution>) {
        self.solver0.initialize(problem.clone(), solution);
        self.problem = problem;
    }

    fn solve(&mut self) -> Solution {
        self.solver1
            .initialize(self.problem.clone(), Some(self.solver0.solve()));
        self.solver1.solve()
    }
}

impl Chain {
    pub fn new(solver0: Box<dyn Solver>, solver1: Box<dyn Solver>) -> Self {
        Chain {
            solver0,
            solver1,
            problem: Problem::default(),
        }
    }
}
