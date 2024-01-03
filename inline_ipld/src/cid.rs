//! Helpers for working with [`Cid`]s

use crate::codec::EncodableAs;
use libipld::{
    cid,
    cid::Cid,
    cid::CidGeneric,
    codec::{Codec, Encode},
    ipld::Ipld,
};
use multihash::MultihashDigest;

/// Create a [`Cid`] for some [`Ipld`]
///
/// # Arguments
///
/// * `ipld`     - [`Ipld`] to create the [`Cid`] for          
/// * `codec`    - The [`Codec`] the [`Ipld`] is encoded with  
/// * `digester` - The [`MultihashDigest`] used in the [`Cid`]
/// * `version`  - The [`Cid`] version                       
///
/// # Examples
///
/// ```
/// # use inline_ipld::cid;
/// # use libipld::{ipld, cid::Version};
/// # use libipld::cbor::DagCborCodec;
/// # use multihash::Code::Sha2_256;
/// # use std::{collections::BTreeMap, str::FromStr};
/// #
/// let observed = cid::new(&ipld!([1, 2, 3]), DagCborCodec, &Sha2_256, Version::V1);
/// let expected = FromStr::from_str("bafyreickxqyrg7hhhdm2z24kduovd4k4vvbmfmenzn7nc6pxg6qzjm2v44").unwrap();
/// assert_eq!(observed, expected);
/// ```
pub fn new<C: Codec, I: EncodableAs<C>, D: MultihashDigest<64>>(
    ipldish: &I,
    codec: C,
    digester: &D,
    version: cid::Version,
) -> Cid
where
    Ipld: Encode<C>,
{
    let encoded: Vec<u8> = ipldish.encodable_as(codec).guaranteed_encode();
    let multihash = digester.digest(&encoded);
    CidGeneric::new(version, codec.into(), multihash)
        .expect("should not fail unless `EncodableAs` is improperly implemented for your codec")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{cid_config::CidConfig, super_ipld::SuperIpld};
    use libipld::cbor::DagCborCodec;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_build_prop_test((SuperIpld(ipld), CidConfig {digester, version, ..}) in (any::<SuperIpld>(), any::<CidConfig>())) {
            let observed = new(&ipld, DagCborCodec, &digester, version);
            prop_assert_eq!(observed.codec(), <u64>::from(DagCborCodec));
            prop_assert_eq!(observed.version(), version);
        }
    }
}
