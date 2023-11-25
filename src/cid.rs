use libipld::{
    cid,
    cid::Cid,
    cid::CidGeneric,
    codec::{Codec, Encode},
    ipld::Ipld,
};
use multihash::MultihashDigest;
use thiserror::Error;

pub fn new<C: Codec, D: MultihashDigest<64>>(
    ipld: &Ipld,
    codec: C,
    digester: &D,
    version: cid::Version,
) -> Result<Cid, Error>
where
    Ipld: Encode<C>,
{
    let encoded = codec.encode(ipld).map_err(Error::EncodingError)?;
    let multihash = digester.digest(encoded.as_slice());
    CidGeneric::new(version, codec.into(), multihash).map_err(Error::ConstructionError)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    EncodingError(#[from] libipld::error::Error),

    #[error("unable to convert to `Cid`")]
    ConstructionError(#[from] cid::Error),
}
