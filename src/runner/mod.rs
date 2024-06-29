mod chain;
mod command;
mod problem;
mod solution;
mod solver;

use once_cell::sync::Lazy;
use std::collections::HashMap;

pub use chain::Chain;
pub use command::SolveCommand;
pub use problem::Problem;
pub use solution::Solution;
pub use solver::{NoopSolver, Parameter, Solver};

use crate::spaceship::{SpaceshipGreedy, SpaceshipOneByOne};

static SOLVERS: Lazy<HashMap<&'static str, Box<dyn Solver>>> = Lazy::new(|| {
    let mut solvers: HashMap<&'static str, Box<dyn Solver>> = HashMap::new();
    solvers.insert("noop", Box::<NoopSolver>::default());
    solvers.insert("ss:greedy", Box::<SpaceshipGreedy>::default());
    solvers.insert("ss:one_by_one", Box::<SpaceshipOneByOne>::default());
    // Add more solvers here
    solvers
});

pub fn create_solver(solver_spec: &str) -> Box<dyn Solver> {
    if solver_spec.contains('+') {
        let mut solvers = solver_spec.split('+').map(create_individual_solver);
        let chain = Box::new(Chain::new(solvers.next().unwrap(), solvers.next().unwrap()));
        solvers.fold(chain, |chain, next| Box::new(Chain::new(chain, next)))
    } else {
        create_individual_solver(solver_spec)
    }
}

fn create_individual_solver(solver_name: &str) -> Box<dyn Solver> {
    let (solver_name, parameters) = if solver_name.contains('{') {
        assert!(
            solver_name.ends_with('}'),
            "Invalid solver name {}",
            solver_name
        );
        let (solver_name, rest) = solver_name.split_at(solver_name.find('{').unwrap());
        let rest = &rest[1..rest.len() - 1];
        let parameters = rest
            .split(',')
            .map(|s| {
                let (name, value) = s.split_at(
                    s.find('=')
                        .expect("Invalid parameter format, expected `name=value`"),
                );
                (
                    name.to_owned(),
                    if value.chars().nth(1).unwrap().is_ascii_digit() {
                        Parameter::Int(
                            value[1..]
                                .parse::<i64>()
                                .expect("Failed to parse solver parameter as i64"),
                        )
                    } else {
                        Parameter::String(value[1..].to_owned())
                    },
                )
            })
            .collect::<HashMap<_, _>>();
        (solver_name, parameters)
    } else {
        (solver_name, HashMap::new())
    };

    let mut solver: Box<dyn Solver> = SOLVERS.get(solver_name).expect("Unknown solver").clone();
    solver.set_parameters(parameters);
    solver
}
