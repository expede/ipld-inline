use super::{some_codec::SomeCodec, super_multihash::SuperMultihash};
use libipld::{cid, cid::multihash::Code, codec_impl::IpldCodec};
use proptest::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct CidConfig {
    pub codec: IpldCodec,
    pub digester: Code,
    pub version: cid::Version,
}

impl Arbitrary for CidConfig {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        prop_oneof![
            // FIXME IPLD encoding in libipld always fails with DagPb, so generic CID v0 fails
            // any::<SuperMultihash>().prop_map(|SuperMultihash(digester)| CidConfig {
            //     digester: digester,
            //     codec: IpldCodec::DagPb,
            //     version: cid::Version::V0,
            // }),
            (any::<SuperMultihash>(), any::<SomeCodec>()).prop_map(
                |(SuperMultihash(digester), SomeCodec(codec))| {
                    CidConfig {
                        codec,
                        digester,
                        version: cid::Version::V1,
                    }
                }
            ),
        ]
        .boxed()
    }
}
