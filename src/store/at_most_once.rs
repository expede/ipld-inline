use super::traits::Store;
use libipld::{error::BlockNotFound, Cid, Ipld};
use std::cell::RefCell;
use std::collections::hash_set::HashSet;

#[derive(Clone, Debug)]
pub struct AtMostOnceStore<S: Store> {
    pub store: S,
    pub tombstones: RefCell<HashSet<Cid>>,
}

impl<S: Store> From<S> for AtMostOnceStore<S> {
    fn from(store: S) -> Self {
        AtMostOnceStore {
            store,
            tombstones: RefCell::new(HashSet::new()),
        }
    }
}

impl<S: Store> Store for AtMostOnceStore<S> {
    fn get(&self, cid: &Cid) -> Result<&Ipld, BlockNotFound> {
        if self.tombstones.borrow().contains(cid) {
            return Err(BlockNotFound(*cid));
        }

        self.store.get(cid).map(|ipld| {
            self.tombstones.borrow_mut().insert(*cid);
            ipld
        })
    }

    fn put_keyed(&mut self, cid: Cid, ipld: Ipld) {
        self.store.put_keyed(cid, ipld);
    }
}
