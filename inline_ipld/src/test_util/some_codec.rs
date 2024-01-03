use libipld::codec_impl::IpldCodec;
use proptest::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SomeCodec(pub IpldCodec);

impl SomeCodec {
    pub fn new(codec: IpldCodec) -> Self {
        SomeCodec(codec)
    }
}

impl From<IpldCodec> for SomeCodec {
    fn from(codec: IpldCodec) -> Self {
        SomeCodec(codec)
    }
}

impl From<SomeCodec> for IpldCodec {
    fn from(wrapper: SomeCodec) -> Self {
        wrapper.0
    }
}

impl Default for SomeCodec {
    fn default() -> Self {
        SomeCodec(IpldCodec::DagCbor)
    }
}

impl Arbitrary for SomeCodec {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        prop_oneof![
            Just(IpldCodec::Raw),
            Just(IpldCodec::DagPb),
            Just(IpldCodec::DagCbor),
            Just(IpldCodec::DagJson),
        ]
        .prop_map(SomeCodec)
        .boxed()
    }
}
