use super::traits::Store;
use libipld::error::BlockNotFound;
use libipld::{Cid, Ipld};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
pub struct MemoryStore {
    pub store: BTreeMap<Cid, Ipld>,
}

impl Store for MemoryStore {
    fn get(&self, cid: &Cid) -> Result<&Ipld, BlockNotFound> {
        self.store.get(cid).ok_or(BlockNotFound(*cid))
    }

    fn put_keyed(&mut self, cid: Cid, ipld: Ipld) {
        self.store.insert(cid, ipld);
    }
}
