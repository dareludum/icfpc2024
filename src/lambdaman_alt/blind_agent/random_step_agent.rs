use crate::geometry::Move;

use super::Agent;

pub struct RandomStepAgent {
    seed: u32,
    rng_state: u64,
}

const FACTOR: u64 = 1664525u64;
const ADD: u64 = 1013904223u64;
const MODULO: u64 = 4294967296u64;

impl Agent for RandomStepAgent {
    fn new(seed: u32) -> Self {
        Self {
            seed,
            rng_state: seed as u64,
        }
    }

    fn play(&mut self) -> Move {
        let op = match (self.rng_state / 42) % 4 {
            0 => Move::Up,
            1 => Move::Left,
            2 => Move::Down,
            3 => Move::Right,
            _ => panic!(),
        };
        self.rng_state = (FACTOR.wrapping_mul(self.rng_state) + ADD) % MODULO;
        op
    }

    fn emit_agent(self, iter_count: usize) -> String {
        format!(
            r#"
            "let rec generate_instructions seed num_insn = (
                if num_insn < 1 {{
                    ""
                }} else {{
                    (("ULDR" drop ((seed / 42) % 4)) take 1)
                    . (generate_instructions (({FACTOR} * seed + {ADD}) % {MODULO}) (num_insn - 1))
                }}
            ); in generate_instructions {} {iter_count}
        "#,
            self.seed
        )
    }

    fn name() -> &'static str {
        "random_step"
    }
}
