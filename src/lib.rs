pub mod cid;
pub mod extractor;
pub mod inliner;
pub mod iterator;
pub mod store;

use crate::inliner::Inliner;
use crate::iterator::post_order::PostOrderIpldIter;
use crate::store::Store;
use libipld::cid::Version;
use libipld::codec_impl::IpldCodec;
use libipld::Ipld;
use multihash::Code::Sha2_256;
use std::fmt::Debug;

// FIXME: this does its best, basically
// pub fn inline<S: Store + Debug>(ipld: Ipld, store: &S) -> Ipld {
//     let mut po = PostOrderIpldIter::from(ipld);
//
// todo!();
//     // Inliner::new(&mut po, store).do_your_best()
//     //  .last()
//     //  .expect("should at least have the Ipld that was passed in")
//     //  .expect("always returns Ipld at the final step")
// }

pub fn extract<S: Store + Default>(ipld: Ipld) -> S {
    let mut store: S = Default::default();
    store.extract(ipld, IpldCodec::DagCbor, &Sha2_256, Version::V1);
    store
}
