use std::collections::{HashMap, HashSet, VecDeque};

use super::model::{Cell, Grid, Path, Point};

// Naive go to the nearest fruit algorithm
#[allow(dead_code)]
pub fn naive_solution(mut grid: Grid) -> Path {
    let fruits = grid.count(Cell::Fruit);
    let mut path = Path::new(grid.start_post());
    let mut pos = grid.start_post();
    for _ in 0..fruits {
        let (next, next_path) = grid.nearest(pos, Cell::Fruit);
        path.extend(next_path);
        grid.set(next, Cell::Empty);
        pos = next;
    }
    path
}

// Gets out of control after 2nd problem
#[allow(dead_code)]
pub fn bfs_shortest_path_solution(grid: Grid) -> Path {
    let fruit_cells: HashSet<Point> = grid
        .iterate_cells()
        .filter(|(_, cell)| *cell == Cell::Fruit)
        .map(|(point, _)| point)
        .collect();

    // Every time a cell is visited, all the fruits collected along the path are stored in this hashset.
    // If a cell is visited with an already visited set/subset of fruits, the path is not explored further.
    let mut visited_cells: HashMap<Point, HashSet<Vec<Point>>> = grid
        .iterate_cells()
        .map(|(point, _)| (point, HashSet::new()))
        .collect();
    // Each point has a set of sets of points (fruits only) that have been visited before reaching it
    let mut queue = VecDeque::new();
    queue.push_back((grid.start_post(), Vec::new(), Path::new(grid.start_post())));
    while !queue.is_empty() {
        let (point, mut visited, path) = queue.pop_front().unwrap();
        if fruit_cells.contains(&point) && !visited.contains(&point) {
            visited.push(point);
            visited.sort();
        }

        if visited.len() == fruit_cells.len() {
            return path;
        }

        let cell_visited = visited_cells.get_mut(&point).unwrap();
        if cell_visited.contains(&visited) {
            continue;
        } else {
            // let mut sub_visited = visited.clone();
            // while sub_visited.len() > 0 && !cell_visited.contains(&sub_visited) {
            //     cell_visited.insert(sub_visited.clone());
            //     sub_visited.pop();
            // }
            cell_visited.insert(visited.clone());
        }

        visited_cells
            .get_mut(&point)
            .unwrap()
            .insert(visited.clone());

        for (next, move_) in grid.neighbours(point) {
            if grid.get(next) == Cell::Wall {
                continue;
            }

            let mut next_path = path.clone();
            next_path.push(move_);
            queue.push_back((next, visited.clone(), next_path));
        }
    }

    panic!("No solution found");
}
