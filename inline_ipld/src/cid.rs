//! Helpers for working with [`Cid`]s

use crate::ipld::encodable::EncodableAs;
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
    ipld: &I,
    codec: C,
    digester: &D,
    version: cid::Version,
) -> Cid
where
    Ipld: Encode<C>,
{
    let encoded: Vec<u8> = ipld.to_encodable_as(codec).guaranteed_encode();
    let multihash = digester.digest(&encoded);
    CidGeneric::new(version, codec.into(), multihash)
        .expect("FIXME: proptests; pretty sure this can't fail now")
}

// FIXME delete? /// Unable to construct a [`Cid`]
// #[derive(Debug, Error)]
// #[error("unable to generate to `Cid`")]
// pub struct ConstructionError {
//     source: cid::Error,
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{cid_config::CidConfig, super_ipld::SuperIpld};
    use libipld::cbor::DagCborCodec;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn build_prop_test((SuperIpld(ipld), CidConfig {digester, version, ..}) in (any::<SuperIpld>(), any::<CidConfig>())) {
            // FIXME shod be geeric over codecs
            let observed = new(&ipld, DagCborCodec, &digester, version);
            // FIXME remove? prop_assert!(&observed.is_ok());

            // FIXME remove? let unwrapped = &observed.unwrap();
            // FIXME original: prop_assert_eq!(observed.codec(), <IpldCodec as TryInto<u64>>::try_into(codec).unwrap());
            prop_assert_eq!(observed.codec(), <u64>::from(DagCborCodec));
            prop_assert_eq!(observed.version(), version);
        }
    }

    // FIXME eliminated completely?
    // // The Raw codec ONLY works on raw byte streams, which is kind of like a base case.
    // // Sadly this completely breaks the generic encoding contract:
    // //
    // // ```
    // // where
    // //   Ipld: Encode<C>,
    // // ```
    // //
    // // I wish this was more typesafe ¯\_(ツ)_/¯
    // #[test]
    // fn raw_encode_test() {
    //     // FIXME maybe not wrap in Ipld?
    //     let dag = ipld!(vec![1_u8, 2_u8, 3_u8]);
    //     let observed = new(&dag, DagCborCodec, &Sha2_256, cid::Version::V1);
    //     assert!(observed.is_ok());
    // }

    // FIXME resurrect when EncodableAs<DagPbCodec> support works
    // #[test]
    // fn pb_encode_test() {
    //     let dag = ipld!([1, 2, 3]);
    //     IpldCodec::DagPb.encode(&dag).unwrap();

    //     assert!(true);
    // }
}
