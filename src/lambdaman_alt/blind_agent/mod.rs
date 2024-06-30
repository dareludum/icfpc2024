use std::marker::PhantomData;

use crate::{
    geometry::Move,
    icfp::serialize_str,
    runner::{Problem, Solution, Solver},
};

use super::model::LambdamanModel;

mod random_step_agent;

pub use random_step_agent::RandomStepAgent;

pub trait Agent {
    fn name() -> &'static str;
    fn new(seed: u32) -> Self;
    fn play(&mut self) -> Move;
    fn emit_agent(self, move_count: usize) -> String;
}

struct Attempt {
    code: String,
    model: LambdamanModel,
}

pub struct BlindAgentSolver<T: Agent> {
    problem: Problem,
    model: LambdamanModel,
    _t: PhantomData<fn() -> T>,
}

// for some reason, macros for Default and Clone think T has to have all the trait implemented too
impl<T: Agent> Default for BlindAgentSolver<T> {
    fn default() -> Self {
        Self {
            problem: Default::default(),
            model: Default::default(),
            _t: Default::default(),
        }
    }
}

impl<T: Agent> Clone for BlindAgentSolver<T> {
    fn clone(&self) -> Self {
        Self {
            problem: self.problem.clone(),
            model: self.model.clone(),
            _t: self._t,
        }
    }
}

impl<T: Agent> BlindAgentSolver<T> {
    fn attempt(&self, seed: u32, max_moves: usize) -> Attempt {
        let mut model = self.model.clone();
        let mut agent = T::new(seed);
        for _ in 0..max_moves {
            model.apply(agent.play());
            if model.is_solved() {
                break;
            }
        }

        Attempt {
            code: agent.emit_agent(model.move_count),
            model,
        }
    }
}

impl<T: Agent> Solver for BlindAgentSolver<T> {
    fn name(&self) -> String {
        format!("lm:{}", T::name())
    }

    fn initialize(&mut self, problem: Problem, _solution: Option<Solution>) {
        self.problem = problem;
        self.model = LambdamanModel::load(&self.problem.load());
    }

    fn solve(&mut self) -> Solution {
        let mut best_attempt: Option<Attempt> = None;

        for _ in 0..1000 {
            let seed: u32 = rand::random();
            let attempt = self.attempt(seed, 1_000_000);

            if attempt.model.is_solved() {
                best_attempt = Some(attempt);
                break;
            }

            if let Some(best_attempt) = best_attempt.as_mut() {
                if attempt.model.fruit_count < best_attempt.model.fruit_count {
                    *best_attempt = attempt;
                }
            } else {
                best_attempt = Some(attempt);
            }
        }

        let best_attempt = best_attempt.unwrap();

        if !best_attempt.model.is_solved() {
            eprintln!(
                "no solution found. best run has {} remaining items:",
                best_attempt.model.fruit_count
            );
            best_attempt.model.print();
            std::process::exit(1);
        }

        let problem_name = self.problem.name.as_str();
        let code = format!(r#""solve {problem_name} " . {}"#, best_attempt.code);

        eprintln!("found solution:\n{code}");

        let node = match crate::lasm::parse(&code) {
            Ok(node) => node,
            Err(err) => {
                eprintln!(
                    "Parsing failed:\n{}",
                    nom::error::convert_error(code.as_str(), err)
                );
                panic!();
            }
        };
        let node = crate::lasm::compile(node);
        Solution::new(node.clone(), serialize_str(node).len() as u64)
    }
}
