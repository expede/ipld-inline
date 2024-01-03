use crate::{cid, codec::Total, test_util::cid_config::CidConfig};
use libipld::Ipld;
use proptest::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct SuperIpld(pub Ipld);

impl Arbitrary for SuperIpld {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        let leaf = prop_oneof![
            Just(Ipld::Null),
            any::<bool>().prop_map(Ipld::Bool),
            any::<Vec<u8>>().prop_map(Ipld::Bytes),
            any::<i128>().prop_flat_map(move |i| {
                any::<codec::Total>().prop_map(move |codec| match codec {
                    codec::Total::DagCbor(_) => Ipld::Integer((i as i64).into()),
                    codec::Total::DagJson(_) => Ipld::Integer((i % (2 ^ 53)).into()), // RAGE
                })
            }),
            any::<f64>().prop_map(Ipld::Float),
            ".*".prop_map(Ipld::String),
            // We don't deref these Links, so just use numbers
            any::<(u64, CidConfig)>().prop_map(
                |(
                    some_u64,
                    CidConfig {
                        digester,
                        version,
                        codec,
                    },
                )| Ipld::Link(cid::new(
                    &Ipld::Integer(some_u64 as i128),
                    codec,
                    &digester,
                    version
                ))
            )
        ];

        let coll = leaf.clone().prop_recursive(16, 1024, 128, |inner| {
            prop_oneof![
                prop::collection::vec(inner.clone(), 0..128).prop_map(Ipld::List),
                prop::collection::btree_map(".*", inner, 0..128).prop_map(Ipld::Map),
            ]
        });

        prop_oneof![
            1 => leaf,
            9 => coll
        ]
        .prop_map(SuperIpld)
        .boxed()
    }
}
