use std::{path::Path, rc::Rc};

use crate::ast::Node;

#[derive(Debug, Clone)]
pub struct Solution {
    pub icfp_code: Rc<Node>,
}

impl Solution {
    pub fn new(icfp_code: Rc<Node>) -> Self {
        Self { icfp_code }
    }

    pub fn score(&self) -> u64 {
        self.icfp_code.to_string().len() as u64
    }

    pub fn save(&self, current_solution_path: &Path) {
        std::fs::write(current_solution_path, self.icfp_code.to_string())
            .expect("Failed to write the solution file");
    }
}
