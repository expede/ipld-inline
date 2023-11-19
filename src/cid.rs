use libipld::{
    cid,
    cid::Cid,
    cid::CidGeneric,
    codec::{Codec, Encode},
    ipld::Ipld,
};
use multihash::MultihashDigest;
use thiserror::Error;

pub fn cid_of<C: Codec, D: MultihashDigest<64>>(
    ipld: &Ipld,
    codec: C,
    digester: D,
    version: cid::Version,
) -> Result<Cid, CidError>
where
    Ipld: Encode<C>,
{
    let encoded = codec.encode(ipld).map_err(CidError::EncodingError)?;
    let multihash = digester.digest(encoded.as_slice());
    CidGeneric::new(version, codec.into(), multihash).map_err(CidError::ConstructionError)
}

#[derive(Debug, Error)]
pub enum CidError {
    #[error("unable to encode to multihash: {0}")]
    EncodingError(libipld::error::Error),

    #[error("unable to convert to `Cid`")]
    ConstructionError(cid::Error),
}

impl From<cid::Error> for CidError {
    fn from(err: cid::Error) -> Self {
        CidError::ConstructionError(err)
    }
}
