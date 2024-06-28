use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct Problem {
    pub path: PathBuf,
    pub name: String,
}

impl Problem {
    pub fn new(path: PathBuf, name: String) -> Self {
        Self { path, name }
    }

    pub fn load(&self) -> String {
        std::fs::read_to_string(&self.path).expect("Failed to read the problem file")
    }
}
