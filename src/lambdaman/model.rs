use std::collections::{HashMap, VecDeque};
use std::iter::successors;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Point {
    x: usize,
    y: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Empty,
    Fruit,
    Wall,
    Start,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Move {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Debug, Hash)]
pub struct Path {
    moves: Vec<Move>,
    start_pos: Point,
}

impl Path {
    pub fn new(start_pos: Point) -> Self {
        Self {
            moves: Vec::new(),
            start_pos,
        }
    }

    pub fn extend(&mut self, other: Self) {
        self.moves.extend(other.moves);
    }

    pub fn len(&self) -> usize {
        self.moves.len()
    }

    pub fn is_empty(&self) -> bool {
        self.moves.is_empty()
    }

    pub fn push(&mut self, m: Move) {
        self.moves.push(m);
    }

    pub fn iter_moves(&self) -> std::slice::Iter<Move> {
        self.moves.iter()
    }

    pub fn iter_positions<'a>(&'a self) -> impl Iterator<Item = (Point, usize)> + 'a {
        successors(Some((self.start_pos, 0)), |&(p, i)| {
            if i >= self.moves.len() {
                return None;
            } else {
                return Some((p.apply(self.moves[i]), i + 1));
            }
        })
    }
}

impl Point {
    // No bounds protection
    pub fn apply(self, m: Move) -> Self {
        match m {
            Move::Up => Self {
                x: self.x,
                y: self.y - 1,
            },
            Move::Down => Self {
                x: self.x,
                y: self.y + 1,
            },
            Move::Left => Self {
                x: self.x - 1,
                y: self.y,
            },
            Move::Right => Self {
                x: self.x + 1,
                y: self.y,
            },
        }
    }
}

impl From<char> for Cell {
    fn from(c: char) -> Self {
        match c {
            ' ' => Self::Empty,
            '.' => Self::Fruit,
            '#' => Self::Wall,
            'L' => Self::Start,
            _ => panic!("invalid cell: {}", c),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grid {
    width: usize,
    height: usize,
    stride: usize,
    start_pos: Point,
    data: Vec<Cell>,
}

impl Grid {
    pub fn new(text: &str) -> Self {
        let mut width = 0;
        let mut height = 0;
        let mut start_pos = None;
        let mut data = Vec::new();
        for line in text.lines() {
            if line.is_empty() {
                break;
            }
            if (width > 0) && (line.len() != width) {
                panic!("inconsistent line length");
            }
            width = line.len();
            data.extend(line.chars().map(|c| Cell::from(c)));
            height += 1;
            if line.find('L').is_some() {
                let x = line.find('L').unwrap();
                let y = height - 1;
                start_pos = Some(Point { x, y });
            }
        }
        let stride = width;
        Self {
            width,
            height,
            stride,
            start_pos: start_pos.unwrap(),
            data,
        }
    }

    pub fn get(&self, p: Point) -> Cell {
        self.internal_get(p.x, p.y)
    }

    fn internal_get(&self, x: usize, y: usize) -> Cell {
        self.data[y * self.stride + x]
    }

    pub fn set(&mut self, p: Point, c: Cell) {
        self.data[p.y * self.stride + p.x] = c;
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn start_post(&self) -> Point {
        self.start_pos
    }

    pub fn count(&self, cell: Cell) -> usize {
        self.data.iter().filter(|&&c| c == cell).count()
    }

    pub fn neighbours(&self, p: Point) -> Vec<(Point, Move)> {
        // caching? trait?
        let mut res = Vec::new();
        if p.x > 0 && self.internal_get(p.x - 1, p.y) != Cell::Wall {
            res.push((Point { x: p.x - 1, y: p.y }, Move::Left));
        }
        if p.x + 1 < self.width && self.internal_get(p.x + 1, p.y) != Cell::Wall {
            res.push((Point { x: p.x + 1, y: p.y }, Move::Right));
        }
        if p.y > 0 && self.internal_get(p.x, p.y - 1) != Cell::Wall {
            res.push((Point { x: p.x, y: p.y - 1 }, Move::Up));
        }
        if p.y + 1 < self.height && self.internal_get(p.x, p.y + 1) != Cell::Wall {
            res.push((Point { x: p.x, y: p.y + 1 }, Move::Down));
        }
        res
    }

    // Find nearest cell of a given type
    pub fn nearest(&self, src: Point, cell: Cell) -> (Point, Path) {
        let mut visited = vec![false; self.data.len()];
        let mut queue = VecDeque::from([src]);
        let mut moves = vec![Vec::<Move>::new(); self.data.len()];
        visited[src.y * self.stride + src.x] = true;
        while let Some(p) = queue.pop_front() {
            for (n, m) in self.neighbours(p) {
                let idx = n.y * self.stride + n.x;
                if visited[idx] {
                    continue;
                }
                visited[idx] = true;
                queue.push_back(n);
                moves[idx] = moves[p.y * self.stride + p.x].clone();
                moves[idx].push(m);
                if self.get(n) == cell {
                    return (
                        n,
                        Path {
                            moves: moves[idx].clone(),
                            start_pos: src,
                        },
                    );
                }
            }
        }
        panic!("no path found");
    }

    // Doesn't work with intersections
    pub fn print_path_to_string(&self, src: Point, path: &Path) -> String {
        let mut special_cells = HashMap::<Point, char>::new();
        let mut pos = src;

        for m in path.iter_moves() {
            special_cells.insert(
                pos,
                match m {
                    Move::Up => '↑',
                    Move::Down => '↓',
                    Move::Left => '←',
                    Move::Right => '→',
                },
            );

            pos = pos.apply(*m);
        }

        special_cells.insert(pos, 'X');

        let mut res = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                res.push(match special_cells.get(&Point { x, y }) {
                    Some(c) => *c,
                    None => match self.get(Point { x, y }) {
                        Cell::Empty => ' ',
                        Cell::Fruit => '.',
                        Cell::Wall => '#',
                        Cell::Start => 'L',
                    },
                });
            }
            res.push('\n');
        }
        res
    }

    pub fn print_highlight_cell_to_string(&self, highlight: Point) -> String {
        let mut res = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                res.push(match (Point { x, y }) == highlight {
                    true => 'X',
                    false => match self.get(Point { x, y }) {
                        Cell::Empty => ' ',
                        Cell::Fruit => '.',
                        Cell::Wall => '#',
                        Cell::Start => 'L',
                    },
                });
            }
            res.push('\n');
        }
        res
    }

    pub fn iterate_cells<'a>(&'a self) -> impl Iterator<Item = (Point, Cell)> + 'a {
        (0..self.data.len()).map(move |i| {
            let x = i % self.stride;
            let y = i / self.stride;
            (Point { x, y }, self.data[i])
        })
    }

    pub fn pathfind_optimal(&self, src: Point, dst: Point) -> Path {
        let mut visited = vec![false; self.data.len()];
        let mut queue = VecDeque::from([src]);
        let mut moves = vec![Vec::<Move>::new(); self.data.len()];
        visited[src.y * self.stride + src.x] = true;
        while let Some(p) = queue.pop_front() {
            for (n, m) in self.neighbours(p) {
                // Neighbours exclude walls
                let idx = n.y * self.stride + n.x;
                if visited[idx] {
                    continue;
                }
                visited[idx] = true;
                queue.push_back(n);
                moves[idx] = moves[p.y * self.stride + p.x].clone();
                moves[idx].push(m);
                if n == dst {
                    return Path {
                        moves: moves[idx].clone(),
                        start_pos: src,
                    };
                }
            }
        }
        panic!("no path found");
    }
}

impl ToString for Grid {
    fn to_string(&self) -> String {
        let mut res = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                res.push(match self.get(Point { x, y }) {
                    Cell::Empty => ' ',
                    Cell::Fruit => '.',
                    Cell::Wall => '#',
                    Cell::Start => 'L',
                });
            }
            res.push('\n');
        }
        res
    }
}

fn max(a: usize, b: usize) -> usize {
    if a > b {
        a
    } else {
        b
    }
}

fn min(a: usize, b: usize) -> usize {
    if a < b {
        a
    } else {
        b
    }
}

pub fn to_lambdaman_path(path: &Path) -> String {
    path.iter_moves()
        .map(|m| match m {
            Move::Up => "U",
            Move::Down => "D",
            Move::Left => "L",
            Move::Right => "R",
        })
        .collect()
}
