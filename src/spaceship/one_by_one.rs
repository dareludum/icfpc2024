use std::{cmp::Ordering, collections::HashMap, rc::Rc};

use memoize::memoize;

use crate::{
    geometry::Vector2D,
    icfp::{Node, Value},
    runner::{Parameter, Problem, Solution, Solver},
    spaceship::model::{Command, SpaceshipState},
};

use super::model::SpaceshipModel;

#[derive(Debug, Clone, Default)]
pub struct SpaceshipOneByOne {
    pub problem: Problem,
    pub model: SpaceshipModel,
    pub start_stop: bool,
}

impl Solver for SpaceshipOneByOne {
    fn name(&self) -> String {
        "one_by_one".to_owned()
    }

    fn initialize(&mut self, problem: Problem, _solution: Option<Solution>) {
        self.problem = problem;
        self.model = SpaceshipModel::load(&self.problem.load());
        self.start_stop = false;
    }

    fn set_parameters(&mut self, parameters: HashMap<String, Parameter>) {
        for (key, value) in parameters {
            match key.as_str() {
                "start_stop" => match value {
                    Parameter::Int(1) => {
                        self.start_stop = true;
                    }
                    _ => {
                        panic!("Invalid value for start_stop: {:?}", value);
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

        let mut points_to_visit = self.model.points.clone();

        while !points_to_visit.is_empty() {
            points_to_visit.sort_by(|a, b| {
                let dist_a = (current_state.pos - *a).manhattan();
                let dist_b = (current_state.pos - *b).manhattan();
                dist_a.cmp(&dist_b)
            });

            println!("Points to visit: {:?}", points_to_visit.len());

            let target_point = points_to_visit[0];
            println!("Target point: {:?}", target_point);

            let path = if self.start_stop {
                let mut path = vec![];

                let strategy_x =
                    find_best_acceleration_strategy(target_point.x - current_state.pos.x);
                let strategy_y =
                    find_best_acceleration_strategy(target_point.y - current_state.pos.y);

                let min_len = strategy_x.len().min(strategy_y.len());
                for i in 0..min_len {
                    let x = strategy_x[i];
                    let y = strategy_y[i];
                    let command = accel_to_command(x, y);
                    current_state = current_state.next(command);
                    path.push(current_state.pos);
                }

                match strategy_x.len().cmp(&strategy_y.len()) {
                    Ordering::Less => {
                        for y in strategy_y.into_iter().skip(strategy_x.len()) {
                            let command = match y {
                                1 => Command::Up,
                                0 => Command::KeepSpeed,
                                -1 => Command::Down,
                                _ => unreachable!(),
                            };
                            current_state = current_state.next(command);
                            path.push(current_state.pos);
                        }
                    }
                    Ordering::Equal => {}
                    Ordering::Greater => {
                        for x in strategy_x.into_iter().skip(strategy_y.len()) {
                            let command = match x {
                                1 => Command::Right,
                                0 => Command::KeepSpeed,
                                -1 => Command::Left,
                                _ => unreachable!(),
                            };
                            current_state = current_state.next(command);
                            path.push(current_state.pos);
                        }
                    }
                }

                assert_eq!(current_state.pos, target_point);
                assert_eq!(current_state.speed, Vector2D::default());

                points_to_visit.remove(0);

                path
            } else {
                let mut tries = 1000;
                let mut path = vec![];
                loop {
                    let mut moves = current_state.get_all_next_moves();
                    moves.sort_by(|a, b| {
                        let dist_a = (a.pos - target_point).manhattan();
                        let dist_b = (b.pos - target_point).manhattan();
                        dist_a.cmp(&dist_b)
                    });

                    current_state = moves.into_iter().next().unwrap();

                    if tries < 1 {
                        let dist_to_target = (current_state.pos - target_point).manhattan();
                        println!(
                            "Dist to target: {}, current pos: {:?}, current speed: {:?}",
                            dist_to_target, current_state.pos, current_state.speed
                        );
                    }

                    path.push(current_state.pos);

                    if current_state.pos == target_point {
                        points_to_visit.remove(0);
                        break;
                    }

                    tries += 1;
                }

                path
            };

            let current_len = points_to_visit.len();
            for point in &path {
                points_to_visit.retain(|p| p != point);
            }
            if points_to_visit.len() < current_len {
                println!("Visited {} points", current_len - points_to_visit.len());
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

fn accel_to_command(x: i32, y: i32) -> Command {
    match (x, y) {
        (1, 1) => Command::UpRight,
        (1, 0) => Command::Up,
        (1, -1) => Command::UpLeft,
        (0, 1) => Command::Right,
        (0, 0) => Command::KeepSpeed,
        (0, -1) => Command::Left,
        (-1, 1) => Command::DownRight,
        (-1, 0) => Command::Down,
        (-1, -1) => Command::DownLeft,
        _ => unreachable!(),
    }
}

#[memoize]
fn find_best_acceleration_strategy(mut distance: i32) -> Vec<i32> {
    match distance {
        0 => return vec![],
        1 => return vec![1, -1],
        -1 => return vec![-1, 1],
        _ => {}
    }

    let mut current_speed = 0;
    let mut result = vec![];

    while distance > 0 {
        let increased_speed = current_speed + 1;
        let minimum_deceleration_distance = increased_speed * (increased_speed + 2) / 2;

        match distance.cmp(&minimum_deceleration_distance) {
            Ordering::Less => {
                if current_speed == 1 {
                    result.push(0);
                } else {
                    current_speed -= 1;
                    result.push(-1);
                }
            }
            Ordering::Equal => {
                result.push(0);
            }
            Ordering::Greater => {
                current_speed += 1;
                result.push(1);
            }
        }

        distance -= current_speed;
    }

    while current_speed > 0 {
        current_speed -= 1;
        result.push(-1);
        distance -= current_speed;
    }
    while current_speed < 0 {
        current_speed += 1;
        result.push(1);
        distance -= current_speed;
    }

    if distance < 0 {
        if current_speed == 1 {
            result.push(-1);
            distance -= 1;
        }
        result.extend(
            find_best_acceleration_strategy(-distance)
                .into_iter()
                .map(|x| -x),
        );
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn x() {
        for i in 0..100000 {
            let strategy = find_best_acceleration_strategy(i);
            check_acceleration_strategy(i, strategy);
        }
    }

    fn check_acceleration_strategy(distance: i32, strategy: Vec<i32>) {
        let mut current_speed = 0;
        let mut current_distance = 0;
        for acceleration in strategy {
            current_speed += acceleration;
            current_distance += current_speed;
        }
        assert!(current_speed == 0);
        assert_eq!(current_distance, distance);
    }
}
