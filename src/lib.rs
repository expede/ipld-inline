pub mod cid;
pub mod extractor;
pub mod inliner;
pub mod iterator;
pub mod store;

// use crate::{extractor::Extractor, inliner::Inliner, store::Store};
// use libipld::codec::{Codec, Encode};
// use libipld::{cid::Version, Ipld};
// use multihash::MultihashDigest;

// FIXME more defaults
// TODO consider making these properties of the store (to and from?)
// FIXME most of this has moved to the store

// pub fn inline<S: Store>(ipld: Ipld, store: S) -> Result<Ipld, inliner::Stuck<S>> {
//     Inliner::new(ipld, store)
//         .next()
//         .expect("should be nonempty")
// }
//
// pub fn extract<C: Codec, D: MultihashDigest<64>>(
//     ipld: Ipld,
//     store: &mut impl Store,
//     codec: C,
//     digester: &D,
//     cid_version: Version,
// ) where
//     Ipld: Encode<C>,
// {
//     for (cid, dag) in Extractor::new(ipld, codec, digester, cid_version) {
//         store.put_keyed(cid, dag);
//     }
// }
