use libipld::cid::multihash::Code;
use proptest::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SuperMultihash(pub Code);

impl SuperMultihash {
    pub fn new(multihash: Code) -> Self {
        SuperMultihash(multihash)
    }
}

impl From<Code> for SuperMultihash {
    fn from(multihash: Code) -> Self {
        SuperMultihash(multihash)
    }
}

impl From<SuperMultihash> for Code {
    fn from(wrapper: SuperMultihash) -> Self {
        wrapper.0
    }
}

impl Arbitrary for SuperMultihash {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        prop_oneof![
            // SHA2
            Just(Code::Sha2_256),
            Just(Code::Sha2_512),
            // SHA3
            Just(Code::Sha3_224),
            Just(Code::Sha3_256),
            Just(Code::Sha3_384),
            Just(Code::Sha3_512),
            // Keccak
            Just(Code::Keccak224),
            Just(Code::Keccak256),
            Just(Code::Keccak384),
            Just(Code::Keccak512),
            // BLAKE2s
            Just(Code::Blake2s128),
            Just(Code::Blake2s256),
            // BLAKE2b
            Just(Code::Blake2b256),
            Just(Code::Blake2b512),
            // BLAKE3
            Just(Code::Blake3_256),
        ]
        .prop_map(SuperMultihash)
        .boxed()
    }
}
