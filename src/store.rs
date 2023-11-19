use crate::cid::{cid_of, CidError};
use libipld::{
    cid,
    cid::Cid,
    codec::{Codec, Encode},
    ipld::Ipld,
};
use multihash::MultihashDigest;
use std::collections::BTreeMap;

// FIXME: unwraps & clones
// FIXME: Docs

pub trait Store: Clone {
    fn get(&self, cid: &Cid) -> Option<&Ipld>;
    fn put_keyed(&mut self, cid: Cid, ipld: Ipld);

    fn put<C: Codec, D: MultihashDigest<64>>(
        &mut self,
        codec: C,
        digester: D,
        version: cid::Version,
        ipld: Ipld,
    ) -> Result<(), CidError>
    where
        Ipld: Encode<C>,
    {
        // FIXME
        let cid = cid_of(&ipld, codec, digester, version)?;
        self.put_keyed(cid, ipld);
        Ok(())
    }
}

#[derive(Clone)]
pub struct MemoryStore {
    store: BTreeMap<Cid, Ipld>,
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

impl Store for MemoryStore {
    fn get(&self, cid: &Cid) -> Option<&Ipld> {
        self.store.get(cid)
    }

    fn put_keyed(&mut self, cid: Cid, ipld: Ipld) {
        self.store.insert(cid, ipld);
    }
}
