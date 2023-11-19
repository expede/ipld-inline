pub mod cid;
pub mod extractor;
pub mod inliner;
pub mod iterator;
pub mod store;

use crate::{extractor::Extractor, store::Store};
// use crate::{extractor::Extractor, inliner::Inliner, store::Store};
use libipld::codec::{Codec, Encode};
use libipld::{cid::Version, Ipld};
use multihash::MultihashDigest;

// FIXME more defaults
// TODO consider making these properties of the store (to and from?)

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

// pub fn inline<'a, S: Store>(ipld: &'a Ipld, store: S) -> inliner::State<'a, S> {
//     Inliner::new(ipld, store).try_inline()
// }
