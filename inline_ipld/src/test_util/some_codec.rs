use libipld::codec_impl::IpldCodec;
use proptest::prelude::*;

#[derive(Clone, Debug, PartialEq)]
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
            // The Raw codec ONLY works on raw byte streams, which is kind of like a base case.
            // Sadly that breaks the encoding contract, so we ignore it here.
            //
            // I wish this was more typesafe.
            //
            // Just(IpldCodec::Raw),

            // FIXME ANNOYING! Can't create CIDs with these codecs. Dafuq
            // Just(IpldCodec::DagPb),
            Just(IpldCodec::DagCbor),
            Just(IpldCodec::DagJson),
        ]
        .prop_map(SomeCodec)
        .boxed()
    }
}
