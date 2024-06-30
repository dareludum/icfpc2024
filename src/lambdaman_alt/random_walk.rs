use crate::{
    geometry::Move,
    icfp::serialize_str,
    runner::{Problem, Solution, Solver},
};

use super::model::LambdamanModel;

#[derive(Debug, Clone, Default)]
pub struct RandomWalk {
    problem: Problem,
    model: LambdamanModel,
}

impl Solver for RandomWalk {
    fn name(&self) -> String {
        "lm:random_walk".to_owned()
    }

    fn initialize(&mut self, problem: Problem, _solution: Option<Solution>) {
        self.problem = problem;
        self.model = LambdamanModel::load(&self.problem.load());
    }

    fn solve(&mut self) -> Solution {
        const FACTOR: u64 = 1664525u64;
        const ADD: u64 = 1013904223u64;
        const MODULO: u64 = 4294967296u64;

        let mut solution: Option<(u64, usize)> = None;

        'attempts: for attempt_i in 0..1000 {
            let seed: u64 = rand::random();
            let mut rng_state = seed;
            let mut model = self.model.clone();
            for iter in 0..1_000_000 {
                let op = match (rng_state / 42) % 4 {
                    0 => Move::Up,
                    1 => Move::Left,
                    2 => Move::Down,
                    3 => Move::Right,
                    _ => panic!(),
                };
                model.apply(op);
                if model.is_solved() {
                    eprintln!(
                        "problem solved with seed {} on attempt {} after {} steps",
                        seed, attempt_i, iter
                    );
                    solution = Some((seed, iter + 1));
                    break 'attempts;
                }
                rng_state = (FACTOR.wrapping_mul(rng_state) + ADD) % MODULO;
            }
        }

        let (good_seed, attempt_count) = solution.expect("no solution found");

        let problem_name = self.problem.name.as_str();
        let code = format!(
            r#"
            "solve {problem_name} " . let rec generate_instructions seed num_insn = (
                if num_insn < 1 {{
                    ""
                }} else {{
                    (("ULDR" drop ((seed / 42) % 4)) take 1)
                    . (generate_instructions (({FACTOR} * seed + {ADD}) % {MODULO}) (num_insn - 1))
                }}
            ); in generate_instructions {good_seed} {attempt_count}
        "#
        );

        eprintln!("{code}");

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
