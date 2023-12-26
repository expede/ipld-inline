//! Helpers for working with [`Cid`]s
use libipld::{
    cid,
    cid::Cid,
    cid::CidGeneric,
    codec::{Codec, Encode},
    ipld::Ipld,
};
use multihash::MultihashDigest;
use thiserror::Error;

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
/// # use libipld_cbor::DagCborCodec;
/// # use multihash::Code::Sha2_256;
/// # use std::{collections::BTreeMap, str::FromStr};
/// #
/// let observed = cid::new(&ipld!([1, 2, 3]), DagCborCodec, &Sha2_256, Version::V1).unwrap();
/// let expected = FromStr::from_str("bafyreickxqyrg7hhhdm2z24kduovd4k4vvbmfmenzn7nc6pxg6qzjm2v44").unwrap();
/// assert_eq!(observed, expected);
/// ```
pub fn new<C: Codec, D: MultihashDigest<64>>(
    ipld: &Ipld,
    codec: C,
    digester: &D,
    version: cid::Version,
) -> Result<Cid, Error>
where
    Ipld: Encode<C>,
{
    let encoded = codec.encode(ipld).map_err(Error::IpldEncodingError)?;
    let multihash = digester.digest(encoded.as_slice());
    CidGeneric::new(version, codec.into(), multihash).map_err(Error::ConstructionError)
}

/// [`Cid`] construction errors
#[derive(Debug, Error)]
pub enum Error {
    /// Error encoding the [`Ipld`] to make a [`Cid`] from
    #[error(transparent)]
    IpldEncodingError(#[from] libipld::error::Error),

    /// Unable to construct a [`Cid`]
    #[error("unable to generate to `Cid`")]
    ConstructionError(#[from] cid::Error),
}
