use std::{path::Path, rc::Rc};

use crate::{ast::Node, serializer::serialize_str};

#[derive(Debug, Clone)]
pub struct Solution {
    pub icfp_code: Rc<Node>,
    pub text: String,
}

impl Solution {
    pub fn new(icfp_code: Rc<Node>) -> Self {
        Self {
            icfp_code: icfp_code.clone(),
            text: serialize_str(icfp_code),
        }
    }

    pub fn score(&self) -> u64 {
        self.text.len() as u64
    }

    pub fn save(&self, current_solution_path: &Path) {
        std::fs::write(current_solution_path, &self.text)
            .expect("Failed to write the solution file");
    }
}
