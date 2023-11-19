pub mod cid;
pub mod extractor;
pub mod inliner;
pub mod iterator;
pub mod store;

use crate::{extractor::Extractor, inliner::Inliner, store::Store};
use libipld::codec::{Codec, Encode};
use libipld::{cid::Version, Ipld};
use multihash::MultihashDigest;

// FIXME more defaults
// TODO consider making these properties of the store (to and from?)

pub fn try_inline_fully<S: Store>(ipld: &Ipld, store: S) -> inliner::State<S> {
    Inliner::new(ipld, store).try_inline()
}

pub fn extract<C: Codec>(
    ipld: &Ipld,
    store: &mut impl Store,
    codec: C,
    digester: impl MultihashDigest<64>,
    cid_version: Version,
) where
    Ipld: Encode<C>,
{
    for (cid, dag) in Extractor::new(ipld, codec, digester, cid_version) {
        store.put_keyed(cid, dag);
    }
}
