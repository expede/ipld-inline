//! Content-addressed store trait
use crate::{cid, extractor::Extractor, ipld::inlined::InlineIpld};
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

/// A trait describing inline/extract-capable content addressed stores
pub trait Store {
    /// Retrieve a block by CID
    ///
    /// # Arguments
    ///
    /// * `self` - The block store
    /// * `cid` - The [`Cid`] to look up by
    ///
    /// # Examples
    ///
    /// ```
    /// # use inline_ipld::store::traits::Store;
    /// # use std::{collections::BTreeMap, str::FromStr};
    /// # use multihash::Code::Sha2_256;
    /// # use libipld::{
    /// #     cid::Version,
    /// #     cbor::DagCborCodec,
    /// #     ipld
    /// # };
    /// #
    /// let block = ipld!([1, 2, 3]);
    /// let mut store = BTreeMap::new();
    /// let cid = store.put(block.clone(), DagCborCodec, &Sha2_256, Version::V1).unwrap();
    ///
    /// assert_eq!(Store::get(&store, &cid).unwrap(), &block);
    /// ```
    fn get(&self, cid: &Cid) -> Result<&Ipld, BlockNotFound>;

    /// Insert a block manually with a user-specified CID
    ///
    /// Since this method _may_ result in a invalid content address, [`Store::put`] should be preferred where possible.
    /// However, [`Store::put_keyed`] is easier to define in a trait implementation.
    ///
    /// # Arguments
    ///
    /// * `self` - The block store
    /// * `cid` - The explicit [`Cid`] to index this block by
    /// * `ipld` - The [`Ipld`] to store
    ///
    /// # Examples
    ///
    /// ```
    /// # use inline_ipld::store::traits::Store;
    /// # use libipld::{cid, ipld};
    /// # use std::{collections::BTreeMap, str::FromStr};
    /// #
    /// # let block = ipld!([1, 2, 3]);
    /// # let cid = FromStr::from_str("bafyreickxqyrg7hhhdm2z24kduovd4k4vvbmfmenzn7nc6pxg6qzjm2v44").unwrap();
    /// #
    /// let mut store = BTreeMap::new();
    /// store.put_keyed(cid, block.clone());
    ///
    /// assert_eq!(Store::get(&store, &cid).unwrap(), &block);
    /// ```
    fn put_keyed(&mut self, cid: Cid, ipld: Ipld);

    /// Insert a block into content addressed storage
    ///
    /// A variant of this method (`put_default`) is available if the `"sha2"` flag is enabled on [`libipld`].
    ///
    /// # Arguments
    ///
    /// * `self` - The block store
    /// * `ipld` - The [`Ipld`] to store
    /// * `codec` - The [`Codec`] that the IPLD is encoded as
    /// * `digester` - The hash function to use when generating the [`Cid`]
    /// * `cid_version` - The [`Cid`] version
    ///
    /// # Examples
    ///
    /// ```
    /// # use inline_ipld::store::traits::Store;
    /// # use std::{collections::BTreeMap, str::FromStr};
    /// # use multihash::Code::Sha2_256;
    /// # use libipld::{
    /// #     cid::Version,
    /// #     cbor::DagCborCodec,
    /// #     ipld
    /// # };
    /// #
    /// let block = ipld!([1, 2, 3]);
    /// let mut store = BTreeMap::new();
    /// let cid = store.put(block.clone(), DagCborCodec, &Sha2_256, Version::V1).unwrap();
    ///
    /// assert_eq!(Store::get(&store, &cid).unwrap(), &block);
    /// ```
    fn put<C: Codec, D: MultihashDigest<64>>(
        &mut self,
        ipld: Ipld,
        codec: C,
        digester: &D,
        cid_version: Version,
    ) -> Result<Cid, cid::Error>
    where
        Ipld: Encode<C>,
    {
        let block_cid = cid::new(&ipld, codec, digester, cid_version)?;
        self.put_keyed(block_cid, ipld);
        Ok(block_cid)
    }

    #[cfg(feature = "sha2")]
    /// [`Store::put`] but defaults to [`Sha2_256`] and [`DagCborCodec`]
    fn put_default(&mut self, ipld: Ipld) -> Result<Cid, cid::Error> {
        use libipld::{
            cbor::DagCborCodec,
            cid::{multihash::Sha2_256, Version},
        };

        self.put(ipld, DagCborCodec, &Sha2_256, Version::V1)
    }

    /// Retrieve a block by CID as a raw vector of bytes.
    ///
    /// # Arguments
    ///
    /// * `self` - The block store
    /// * `cid` - The [`Cid`] to look up by
    ///
    /// # Examples
    ///
    /// ```
    /// #  use inline_ipld::store::traits::Store;
    /// #  use std::{collections::BTreeMap, str::FromStr};
    /// #  use multihash::Code::Sha2_256;
    /// #  use libipld::{
    /// #      cid::Version,
    /// #      cbor::DagCborCodec,
    /// #      ipld
    /// #  };
    /// #
    /// let mut store = BTreeMap::new();
    /// let cid = store.put(ipld!([1, 2, 3]), DagCborCodec, &Sha2_256, Version::V1).unwrap();
    /// let observed = store.get_raw(&cid).unwrap();
    ///
    /// assert_eq!(observed, vec![131, 1, 2, 3]);
    /// ```
    fn get_raw(&self, cid: &Cid) -> Result<Vec<u8>, GetRawError> {
        let ipld = self.get(cid).map_err(GetRawError::NotFound)?;
        let codec_id: u64 = cid.codec();
        let codec: IpldCodec = codec_id.try_into().map_err(GetRawError::UnknownCodec)?;

        let mut buffer = vec![];
        ipld.encode(codec, &mut buffer)
            .map_err(GetRawError::EncodeFailed)?;

        Ok(buffer)
    }

    /// Extract all graphs from inlined IPLD and store them
    ///
    /// # Arguments
    ///
    /// * `self` - Where subgraphs will be stored
    /// * `ipld` - The `Ipld` to extract graphs from
    /// * `codec` - The [`Codec`] to extract with if none is provided by the inline IPLD
    /// * `digester` - The digest (hash) function to use if none is specified in the inlined IPLD
    /// * `cid_version` - The CID version to use is none is specified in the inlined IPLD
    ///
    /// # Examples
    ///
    /// ```
    /// # use inline_ipld::store::traits::Store;
    /// #
    /// # use libipld::{ipld, cid::Version};
    /// # use libipld_cbor::DagCborCodec;
    /// # use multihash::Code::Sha2_256;
    /// # use std::{collections::BTreeMap, str::FromStr};
    /// #
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
        ipld: InlineIpld,
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

impl Store for BTreeMap<Cid, Ipld> {
    fn get(&self, cid: &Cid) -> Result<&Ipld, BlockNotFound> {
        self.get(cid).ok_or(BlockNotFound(*cid))
    }

    fn put_keyed(&mut self, cid: Cid, ipld: Ipld) {
        self.insert(cid, ipld);
    }
}

/// Error cases for [`Store::get_raw`]
#[derive(Debug, Error)]
pub enum GetRawError {
    /// Forwards a (lifted) [BlockNotFound]
    #[error(transparent)]
    NotFound(#[from] BlockNotFound),

    /// Forwards a (lifted) [UnsupportedCodec]
    #[error(transparent)]
    UnknownCodec(#[from] UnsupportedCodec),

    /// Forwards a (lifted) [libipld::error::Error]
    /// Note that these are never comparable
    #[error("failed to encode to bytes")]
    EncodeFailed(#[from] libipld::error::Error),
}

impl PartialEq for GetRawError {
    fn eq(&self, other: &GetRawError) -> bool {
        match (self, other) {
            (
                &GetRawError::NotFound(BlockNotFound(cid_a)),
                &GetRawError::NotFound(BlockNotFound(cid_b)),
            ) => cid_a.eq(&cid_b),
            (
                &GetRawError::UnknownCodec(UnsupportedCodec(codec_a)),
                &GetRawError::UnknownCodec(UnsupportedCodec(codec_b)),
            ) => codec_a.eq(&codec_b),
            // libipld::error::Error is existentially quantified, and not constrained with PartialEq, so false
            _ => false,
        }
    }
}
