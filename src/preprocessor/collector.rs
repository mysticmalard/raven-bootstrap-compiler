use std::error::Error;

enum CollectorRoot {
    Root(String),
}

pub struct Collector {
    root: CollectorRoot,
    body: String,
}

impl Collector {
    pub fn new(path: String) -> Result<Self, Box<dyn Error>> {
        let body = std::fs::read_to_string(&path)?;

        Ok(Self { root: CollectorRoot::Root(path), body })
    }
}

impl Collector {
    pub fn get_body(&self) -> &str {
        &self.body
    }
}