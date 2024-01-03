//! An in-memory [`Ipld`] store
use super::traits::Store;
use libipld::error::BlockNotFound;
use libipld::{Cid, Ipld};
use std::collections::BTreeMap;

/// A basic in-memory [`Ipld`] store
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde-codec", derive(serde::Serialize))]
pub struct MemoryStore {
    store: BTreeMap<Cid, Ipld>,
}

impl MemoryStore {
    /// Initialize an empty [`MemoryStore`]
    #[allow(clippy::must_use_candidate)]
    pub const fn new() -> Self {
        MemoryStore {
            store: BTreeMap::new(),
        }
    }
}

impl From<BTreeMap<Cid, Ipld>> for MemoryStore {
    fn from(store: BTreeMap<Cid, Ipld>) -> Self {
        MemoryStore { store }
    }
}

impl From<MemoryStore> for BTreeMap<Cid, Ipld> {
    fn from(ms: MemoryStore) -> Self {
        ms.store
    }
}

impl Store for MemoryStore {
    fn get(&self, cid: Cid) -> Result<&Ipld, BlockNotFound> {
        Store::get(&self.store, cid)
    }

    fn put_keyed(&mut self, cid: Cid, ipld: Ipld) {
        self.store.put_keyed(cid, ipld);
    }
}