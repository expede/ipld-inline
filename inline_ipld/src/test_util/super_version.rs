use libipld::cid::Version;
use proptest::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SuperVersion(pub Version);

impl SuperVersion {
    pub fn new(version: Version) -> Self {
        SuperVersion(version)
    }
}

impl From<Version> for SuperVersion {
    fn from(version: Version) -> Self {
        SuperVersion(version)
    }
}

impl From<SuperVersion> for Version {
    fn from(wrapper: SuperVersion) -> Self {
        wrapper.0
    }
}

impl Arbitrary for SuperVersion {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        prop_oneof![Just(Version::V0), Just(Version::V1)]
            .prop_map(SuperVersion)
            .boxed()
    }
}
