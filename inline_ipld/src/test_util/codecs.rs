use libipld::codec_impl::IpldCodec;
use libipld::{prelude::Codec, Cid, Ipld};
use libipld::cbor::DagCborCodec;
use multihash::{Code::Sha2_256, MultihashDigest};
use proptest::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct SomeCodec(pub IpldCodec);

impl Arbitrary for SomeCodec {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        prop_oneof![Raw, DagCbor, DagJson, DagPb,]
            .prop_map(SomeCodec)
            .boxed()
    }
}
