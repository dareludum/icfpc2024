use std::rc::Rc;

use crate::{
    icfp::{Node, Value},
    runner::{Problem, Solution, Solver},
    spaceship::model::SpaceshipState,
};

use super::model::SpaceshipModel;

#[derive(Debug, Clone, Default)]
pub struct SpaceshipOneByOne {
    pub problem: Problem,
    pub model: SpaceshipModel,
}

impl Solver for SpaceshipOneByOne {
    fn name(&self) -> String {
        "one_by_one".to_owned()
    }

    fn initialize(&mut self, problem: Problem, _solution: Option<Solution>) {
        self.problem = problem;
        self.model = SpaceshipModel::load(&self.problem.load());
    }

    fn solve(&mut self) -> Solution {
        let mut current_state = SpaceshipState::default();

        let mut points_to_visit = self.model.points.clone();

        while !points_to_visit.is_empty() {
            points_to_visit.sort_by(|a, b| {
                let dist_a = (current_state.pos - *a).manhattan();
                let dist_b = (current_state.pos - *b).manhattan();
                dist_a.cmp(&dist_b)
            });

            println!("Points to visit: {:?}", points_to_visit.len());

            let target_point = points_to_visit[0];

            loop {
                let mut moves = current_state.get_all_next_moves();
                moves.sort_by(|a, b| {
                    let dist_a = (a.pos - target_point).manhattan();
                    let dist_b = (b.pos - target_point).manhattan();
                    dist_a.cmp(&dist_b)
                });

                current_state = moves.into_iter().next().unwrap();

                if current_state.pos == target_point {
                    points_to_visit.remove(0);
                    break;
                }
            }
        }

        println!("Path: {}", current_state.path);

        Solution::new(
            Rc::new(Node::Value(Value::Str(format!(
                "solve {} {}",
                self.problem.name, current_state.path
            )))),
            current_state.path.len() as u64,
        )
    }
}
