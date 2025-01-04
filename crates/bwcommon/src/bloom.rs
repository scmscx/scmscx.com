use serde::{Deserialize, Serialize};
use std::{collections::HashSet, hash::Hasher};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApproximateSet {
    vec: Vec<HashSet<i64>>,
}

impl ApproximateSet {
    pub fn new(size: usize) -> ApproximateSet {
        ApproximateSet {
            vec: vec![HashSet::new(); size],
        }
    }

    pub fn insert(&mut self, text: &str, map_id: i64) {
        let mut hasher = fnv::FnvHasher::default();
        hasher.write(text.as_bytes());

        let len = self.vec.len();
        let d = &mut self.vec[hasher.finish() as usize % len];

        d.insert(map_id);
    }

    pub fn get(&self, text: &str) -> &HashSet<i64> {
        let mut hasher = fnv::FnvHasher::default();
        hasher.write(text.as_bytes());

        &self.vec[hasher.finish() as usize % self.vec.len()]
    }
}
