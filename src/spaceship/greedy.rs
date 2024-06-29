use std::{
    collections::{HashMap, HashSet, VecDeque},
    rc::Rc,
};

use crate::{
    icfp::{Node, Value},
    runner::{Parameter, Problem, Solution, Solver},
    spaceship::model::{Command, SpaceshipState},
};

use super::model::SpaceshipModel;

#[derive(Debug, Clone, Default)]
pub struct SpaceshipGreedy {
    pub problem: Problem,
    pub model: SpaceshipModel,
    pub max_possible_states: u32,
}

impl Solver for SpaceshipGreedy {
    fn name(&self) -> String {
        "ss:greedy".to_string()
    }

    fn initialize(&mut self, problem: Problem, _solution: Option<Solution>) {
        self.model = SpaceshipModel::load(&problem.load());
        self.problem = problem;
        self.max_possible_states = 30_000;
    }

    fn set_parameters(&mut self, parameters: HashMap<String, Parameter>) {
        for (key, value) in parameters {
            match key.as_str() {
                "max_possible_states" => match value {
                    Parameter::Int(value) => {
                        self.max_possible_states = value as u32;
                    }
                    _ => {
                        panic!("Invalid value for max_possible_states: {:?}", value);
                    }
                },
                _ => {
                    panic!("Unknown parameter: {}", key);
                }
            }
        }
    }

    fn solve(&mut self) -> Solution {
        let mut current_state = SpaceshipState::default();

        let mut points_to_visit = self.model.points.iter().cloned().collect::<HashSet<_>>();
        let mut min_x = i32::MAX;
        let mut max_x = i32::MIN;
        let mut min_y = i32::MAX;
        let mut max_y = i32::MIN;
        for point in &points_to_visit {
            min_x = min_x.min(point.x);
            max_x = max_x.max(point.x);
            min_y = min_y.min(point.y);
            max_y = max_y.max(point.y);
        }

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

                if possible_states.len() > self.max_possible_states as usize {
                    eprintln!("Too many possible states, exiting");
                    break;
                }

                // Presort states by distance to closest point
                possible_states.make_contiguous().sort_by(|a, b| {
                    let mut a_min_dist = i32::MAX;
                    for point in &points_to_visit {
                        a_min_dist = a_min_dist.min((*point - a.pos).manhattan());
                    }
                    let mut b_min_dist = i32::MAX;
                    for point in &points_to_visit {
                        b_min_dist = b_min_dist.min((*point - b.pos).manhattan());
                    }
                    a_min_dist.cmp(&b_min_dist)
                });

                let new_state = possible_states.pop_front().unwrap();
                if visited_states.contains(&(new_state.pos, new_state.speed)) {
                    continue;
                }

                // Prune states that are going in the wrong direction
                match new_state.path.chars().last().unwrap() {
                    '7' => {
                        if new_state.pos.x < min_x || new_state.pos.y > max_y {
                            continue;
                        }
                    }
                    '8' => {
                        if new_state.pos.y > max_y {
                            continue;
                        }
                    }
                    '9' => {
                        if new_state.pos.x > max_x || new_state.pos.y > max_y {
                            continue;
                        }
                    }
                    '4' => {
                        if new_state.pos.x < min_x {
                            continue;
                        }
                    }
                    '6' => {
                        if new_state.pos.x > max_x {
                            continue;
                        }
                    }
                    '1' => {
                        if new_state.pos.x < min_x || new_state.pos.y < min_y {
                            continue;
                        }
                    }
                    '2' => {
                        if new_state.pos.y < min_y {
                            continue;
                        }
                    }
                    '3' => {
                        if new_state.pos.x > max_x || new_state.pos.y < min_y {
                            continue;
                        }
                    }
                    _ => {}
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
