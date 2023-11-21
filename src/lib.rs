pub mod cid;
pub mod extractor;
pub mod inliner;
pub mod iterator;
pub mod store;

use crate::inliner::Inliner;
use crate::store::traits::Store;
use libipld::{cid::Version, codec_impl::IpldCodec, Ipld};
use multihash::Code::Sha2_256;

pub fn inline(ipld: Ipld, store: &impl Store) -> Ipld {
    Inliner::new(ipld, store)
        .last()
        .expect("should have at least the `Ipld` that was passed in")
        .expect("should have at least the `Ipld` that was passed in")
}

pub fn extract<S: Store>(ipld: Ipld, store: &mut S) -> &S {
    store.extract(ipld, IpldCodec::DagCbor, &Sha2_256, Version::V1);
    store
}
