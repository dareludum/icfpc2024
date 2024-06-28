use super::model::{Cell, Grid, Path};

// Naive go to the nearest fruit algorithm
pub fn naive_path(mut grid: Grid) -> Path {
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
