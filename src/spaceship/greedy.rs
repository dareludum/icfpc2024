use std::{
    collections::{HashSet, VecDeque},
    rc::Rc,
};

use crate::{
    ast::{Node, Value},
    runner::{Problem, Solution, Solver},
    spaceship::model::{Command, SpaceshipState},
};

use super::model::{SpaceshipModel, Vector2D};

#[derive(Debug, Clone, Default)]
pub struct SpaceshipGreedy {
    pub problem: Problem,
    pub model: SpaceshipModel,
}

impl Solver for SpaceshipGreedy {
    fn name(&self) -> String {
        "ss:greedy".to_string()
    }

    fn initialize(&mut self, problem: Problem, _solution: Option<Solution>) {
        self.model = SpaceshipModel::load(&problem.load());
        self.problem = problem;
    }

    fn solve(&mut self) -> Solution {
        let mut current_state =
            SpaceshipState::new(Vector2D::default(), Vector2D::default(), "".to_string());

        let mut points_to_visit = self.model.points.iter().cloned().collect::<HashSet<_>>();
        while !points_to_visit.is_empty() {
            // Look through all points we can visit from this one and pick a path that exists
            let mut possible_states = VecDeque::new();
            {
                possible_states.push_back(current_state.next(Command::UpLeft));
                possible_states.push_back(current_state.next(Command::Up));
                possible_states.push_back(current_state.next(Command::UpRight));
                possible_states.push_back(current_state.next(Command::Left));
                // // Keep speed - doesn't make sense for starting state
                // possible_states.push_back(current_state.next(Command::KeepSpeed));
                possible_states.push_back(current_state.next(Command::Right));
                possible_states.push_back(current_state.next(Command::DownLeft));
                possible_states.push_back(current_state.next(Command::Down));
                possible_states.push_back(current_state.next(Command::DownRight));
            }

            println!("=======================================================");
            println!("Points to visit: {:?}", points_to_visit.len());
            let mut visited_states = HashSet::new();
            loop {
                println!("Possible states: {:?}", possible_states.len());
                println!("Visited states: {:?}", visited_states.len());
                println!("Points to visit: {:?}", points_to_visit.len());

                let new_state = possible_states.pop_front().unwrap();
                if visited_states.contains(&(new_state.pos, new_state.speed)) {
                    continue;
                }

                if points_to_visit.contains(&new_state.pos) {
                    points_to_visit.remove(&new_state.pos);
                    current_state = new_state;
                    visited_states.clear();
                    possible_states.clear();

                    break;
                }

                visited_states.insert((new_state.pos, new_state.speed));
                {
                    // Up-Left
                    let next_state = new_state.next(Command::UpLeft);
                    if !visited_states.contains(&(next_state.pos, next_state.speed)) {
                        possible_states.push_back(next_state);
                    }
                    // Up
                    let next_state = new_state.next(Command::Up);
                    if !visited_states.contains(&(next_state.pos, next_state.speed)) {
                        possible_states.push_back(next_state);
                    }
                    // Up-Right
                    let next_state = new_state.next(Command::UpRight);
                    if !visited_states.contains(&(next_state.pos, next_state.speed)) {
                        possible_states.push_back(next_state);
                    }
                    // Left
                    let next_state = new_state.next(Command::Left);
                    if !visited_states.contains(&(next_state.pos, next_state.speed)) {
                        possible_states.push_back(next_state);
                    }
                    // Keep speed
                    let next_state = new_state.next(Command::KeepSpeed);
                    if !visited_states.contains(&(next_state.pos, next_state.speed)) {
                        possible_states.push_back(next_state);
                    }
                    // Right
                    let next_state = new_state.next(Command::Right);
                    if !visited_states.contains(&(next_state.pos, next_state.speed)) {
                        possible_states.push_back(next_state);
                    }
                    // Down-Left
                    let next_state = new_state.next(Command::DownLeft);
                    if !visited_states.contains(&(next_state.pos, next_state.speed)) {
                        possible_states.push_back(next_state);
                    }
                    // Down
                    let next_state = new_state.next(Command::Down);
                    if !visited_states.contains(&(next_state.pos, next_state.speed)) {
                        possible_states.push_back(next_state);
                    }
                    // Down-Right
                    let next_state = new_state.next(Command::DownRight);
                    if !visited_states.contains(&(next_state.pos, next_state.speed)) {
                        possible_states.push_back(next_state);
                    }
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
