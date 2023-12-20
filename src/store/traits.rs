use crate::cid;
use crate::extractor::Extractor;
use crate::inliner::at_most_once::AtMostOnce;
use crate::inliner::exactly_once::ExactlyOnce;
use crate::inliner::quiet::Quiet;
use libipld::{
    cid::{Cid, Version},
    codec::{Codec, Encode},
    error::{BlockNotFound, UnsupportedCodec},
    ipld::Ipld,
    IpldCodec,
};
use multihash::MultihashDigest;
use std::collections::BTreeMap;
use thiserror::Error;

pub trait Store {
    fn get(&self, cid: &Cid) -> Result<&Ipld, BlockNotFound>;
    fn put_keyed(&mut self, cid: Cid, ipld: Ipld);

    fn put<C: Codec, D: MultihashDigest<64>>(
        &mut self,
        codec: C,
        digester: &D,
        version: Version,
        ipld: Ipld,
    ) -> Result<(), cid::Error>
    where
        Ipld: Encode<C>,
    {
        // FIXME
        let cid = cid::new(&ipld, codec, digester, version)?;
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

    fn try_inline(&mut self, ipld: Ipld) -> Result<Ipld, Cid> {
        ExactlyOnce::new(ipld, self)
            .last()
            .expect("should have at least the `Ipld` that was passed in")
            .clone()
    }

    fn inline_at_most_once(&mut self, ipld: Ipld) -> Ipld {
        Quiet::new(ipld, self)
            .last()
            .expect("should have at least the `Ipld` that was passed in")
            .expect("should have at least the `Ipld` that was passed in")
            .clone()
    }

    fn try_inline_exactly_once(&mut self, ipld: Ipld) -> Result<Ipld, Cid> {
        // FIXME
        AtMostOnce::new(ipld, self)
            .last()
            .expect("should have at least the `Ipld` that was passed in")
    }

    /// Extract all graphs from inlined IPLD and store them
    ///
    /// # Arguments
    ///
    /// * `self` - Where subgraphs will be stored
    /// * `ipld` - The IPLD to extract graphs from
    /// * `codec` - The codec to extract with if none is provided by the inline IPLD
    /// * `digester` - The digest (hash) function to use if none is specified in the inlined IPLD
    /// * `cid_version` - The CID version to use is none is specified in the inlined IPLD
    ///
    /// # Examples
    ///
    /// ```
    /// use ipld_inline::store::traits::Store;
    ///
    /// use libipld::{ipld, cid::Version};
    /// use libipld_cbor::DagCborCodec;
    /// use multihash::Code::Sha2_256;
    /// use std::collections::BTreeMap;
    /// use std::str::FromStr;
    ///
    /// let inner = ipld!([4, 5, 6]);
    /// let inner_cid = FromStr::from_str("bafyreihscx57i276zr5pgnioa5omevods6eseu5h4mllmow6csasju6eqi").unwrap();
    ///
    /// let outer = ipld!({"a": 123, "b": {"data": [4, 5, 6]}});
    /// let outer_cid = FromStr::from_str("bafyreignkagaefshuw6wloom3qh2mb2ytavv6y3s7sogi7hpeoetb7ejki").unwrap();
    ///
    /// let mut expected = BTreeMap::new();
    /// expected.put_keyed(inner_cid, ipld!([4, 5, 6]));
    /// expected.put_keyed(outer_cid, ipld!({"a": 123, "b": inner_cid}));
    ///
    /// let mut observed = BTreeMap::new();
    /// let inlined = ipld!({"a": 123, "b": {"/": {"data": ipld!([4, 5, 6])}}});
    /// observed.extract(inlined, DagCborCodec, &Sha2_256, Version::V1);
    ///
    /// assert_eq!(observed, expected);
    /// ```
    fn extract<C: Codec, D: MultihashDigest<64>>(
        &mut self,
        ipld: Ipld,
        codec: C,
        digester: &D,
        cid_version: Version,
    ) where
        Ipld: Encode<C>,
    {
        for (cid, dag) in Extractor::new(ipld, codec, digester, cid_version) {
            self.put_keyed(cid, dag);
        }
    }
}

/// Error cases for [`get_raw`][Store::get_raw()].
#[derive(Debug, Error)]
pub enum GetRawError {
    /// Forwards a (lifted) [BlockNotFound]
    #[error(transparent)]
    NotFound(#[from] BlockNotFound),

    /// Forwards a (lifted) [UnsupportedCodec]
    #[error(transparent)]
    UnknownCodec(#[from] UnsupportedCodec),

    /// Forwards a (lifted) [libipld::error::Error]
    #[error("failed to encode to bytes")]
    EncodeFailed(#[from] libipld::error::Error),
}

impl Store for BTreeMap<Cid, Ipld> {
    fn get(&self, cid: &Cid) -> Result<&Ipld, BlockNotFound> {
        self.get(cid).ok_or(BlockNotFound(*cid))
    }

    fn put_keyed(&mut self, cid: Cid, ipld: Ipld) {
        self.insert(cid, ipld);
    }
}
