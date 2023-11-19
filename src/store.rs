mod memory;

use crate::cid::{cid_of, CidError};
use libipld::{
    cid,
    cid::Cid,
    codec::{Codec, Encode},
    error::{BlockNotFound, UnsupportedCodec},
    ipld::Ipld,
    IpldCodec,
};
use memory::MemoryStore;
use multihash::MultihashDigest;
use thiserror::Error;

// FIXME: unwraps & clones
// FIXME: Docs

pub trait Store: Clone {
    fn get(&self, cid: &Cid) -> Result<&Ipld, BlockNotFound>;
    fn put_keyed(&mut self, cid: Cid, ipld: Ipld);

    fn put<C: Codec, D: MultihashDigest<64>>(
        &mut self,
        codec: C,
        digester: D,
        version: cid::Version,
        ipld: Ipld,
    ) -> Result<(), CidError>
    where
        Ipld: Encode<C>,
    {
        // FIXME
        let cid = cid_of(&ipld, codec, digester, version)?;
        self.put_keyed(cid, ipld);
        Ok(())
    }

    fn get_raw(&self, cid: &Cid) -> Result<Vec<u8>, GetRawError> {
        let ipld = self.get(cid).map_err(GetRawError::NotFound)?;
        let codec_id: u64 = cid.codec();
        let codec: IpldCodec = codec_id.try_into().map_err(GetRawError::UnknownCodec)?;

        let mut buffer = vec![];
        ipld.encode(codec, &mut buffer)
            .map_err(GetRawError::EncodeFailed)?;

        Ok(buffer)
    }
}

#[derive(Debug, Error)]
pub enum GetRawError {
    #[error(transparent)]
    NotFound(#[from] BlockNotFound),

    #[error(transparent)]
    UnknownCodec(#[from] UnsupportedCodec),

    #[error("failed to encode to bytes")]
    EncodeFailed(#[from] libipld::error::Error),
}

impl Store for MemoryStore {
    fn get(&self, cid: &Cid) -> Result<&Ipld, BlockNotFound> {
        self.store.get(cid).ok_or(BlockNotFound(*cid))
    }

    fn put_keyed(&mut self, cid: Cid, ipld: Ipld) {
        self.store.insert(cid, ipld);
    }
}
