use libipld::{Cid, Ipld};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct MemoryStore {
    pub store: BTreeMap<Cid, Ipld>,
}

impl MemoryStore {
    pub fn new() -> Self {
        MemoryStore {
            store: BTreeMap::new(),
        }
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        MemoryStore::new()
    }
}
