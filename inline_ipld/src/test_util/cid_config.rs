use super::super_multihash::SuperMultihash;
use crate::codec::Total;
use libipld::{cid, cid::multihash::Code};
use proptest::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CidConfig {
    pub codec: codec::Total,
    pub digester: Code,
    pub version: cid::Version,
}

impl Arbitrary for CidConfig {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        prop_oneof![
            // IPLD encoding in libipld currently always fails with DagPb, so generic CID v0 fails
            // any::<SuperMultihash>().prop_map(|SuperMultihash(digester)| CidConfig {
            //     digester: digester,
            //     codec: IpldCodec::DagPb,
            //     version: cid::Version::V0,
            // }),
            (any::<SuperMultihash>(), any::<codec::Total>()).prop_map(
                |(SuperMultihash(digester), codec)| {
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
