//! Newtype wrapper for [`InlineIpld`]

use crate::cid;
use libipld::{
    cid::Version,
    codec::{Codec, Encode},
    ipld, Cid, Ipld,
};
use multihash::MultihashDigest;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Newtype wrapper for [`InlineIpld`]
///
/// This is helpful to indictate that some form of inlining has already been performed.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct InlineIpld {
    ipld: Ipld,
}

impl PartialEq<Ipld> for InlineIpld {
    fn eq(&self, other: &Ipld) -> bool {
        other.eq(&self.ipld)
    }
}

impl From<InlineIpld> for Ipld {
    fn from(inline: InlineIpld) -> Ipld {
        inline.ipld
    }
}

impl<'a> From<&'a InlineIpld> for &'a Ipld {
    fn from(inline: &'a InlineIpld) -> &'a Ipld {
        &inline.ipld
    }
}

impl InlineIpld {
    /// Wrap some [`Ipld`] and manually associated [`Cid`] in an inline delimiter
    ///
    /// To have the [`Cid`] calculated at runtime, use [`Self::wrap_with_link`].
    /// [`Self::wrap`] *does not* check that the [`Cid`] actually corresponds to the associated [`Ipld`].
    ///
    /// # Arguments
    ///
    /// * `cid` - The [`Cid`] for the `"link"` field
    /// * `ipld` - The [`Ipld`] to inline
    ///
    /// ```
    /// # use inline_ipld::{cid, ipld::inlined::InlineIpld};
    /// # use libipld::{ipld, Ipld, Cid};
    /// # use std::str::FromStr;
    /// #
    /// let cid: Cid = FromStr::from_str("bafyreihscx57i276zr5pgnioa5omevods6eseu5h4mllmow6csasju6eqi").unwrap();
    /// assert_eq!(InlineIpld::wrap(cid, ipld!([1, 2, 3])), ipld!({
    ///   "/": {
    ///     "data": ipld!([1, 2, 3]),
    ///     "link": cid
    ///   }
    /// }));
    /// ```
    pub fn wrap(cid: Cid, ipld: Ipld) -> Self {
        InlineIpld {
            ipld: ipld!({
              "/": {
                "data": ipld!(ipld),
                "link": Ipld::Link(cid)
              }
            }),
        }
    }

    /// Create inline-delimited [`Ipld`], but omit the `"link"` field
    ///
    /// `inline_ipld_inherit` wraps the enclosed [`Ipld`] in the correct delimiters.
    /// It omits the `"link"` field. Per the [spec], this causes the extracted
    /// IPLD link to inherit the coedec and CID encoding of its parent.
    ///
    /// [spec]: https://github.com/ucan-wg/ipld-inline-links
    ///
    /// # Arguments
    ///
    /// * `ipld` - The [`Ipld`] to wrap in the inlined delimiter
    ///
    /// # Examples
    ///
    /// ```
    /// # use inline_ipld::{cid, ipld::inlined::InlineIpld};
    /// # use libipld::{ipld, Ipld};
    /// #
    /// let observed = InlineIpld::wrap_inherit_link(ipld!([1, 2, 3]));
    /// assert_eq!(observed, ipld!({"/": {"data": ipld!([1, 2, 3])}}));
    /// ```
    pub fn wrap_inherit_link(ipld: Ipld) -> Self {
        InlineIpld {
            ipld: ipld!({"/": {"data": ipld!(ipld)}}),
        }
    }

    /// Create inline-delimited [`Ipld`], and automatically calculate the [`Cid`]
    ///
    /// Unlike [`Self::wrap`], [`Self::wrap_with_link`] calculates a correct `"link"` based on
    /// the codec and digest function passed in. It always uses to CIDv1.
    ///
    /// # Arguments
    ///
    /// * `codec` - The codec to encode the [`Ipld`] with when generating the [`Cid`]
    /// * `digester` - The hash function for the [`Cid`]
    /// * `ipld` - The [`Ipld`] to inline
    ///
    /// # Examples
    ///
    /// ```
    /// # use inline_ipld::{cid, ipld::inlined::InlineIpld};
    /// # use std::str::FromStr;
    /// # use multihash::Code::Sha2_256;
    /// # use libipld::{
    /// #     cid::Version,
    /// #     cbor::DagCborCodec,
    /// #     ipld,
    /// #     Ipld,
    /// #     Cid
    /// # };
    /// #
    /// let cid: Cid = FromStr::from_str("bafyreickxqyrg7hhhdm2z24kduovd4k4vvbmfmenzn7nc6pxg6qzjm2v44").unwrap();
    /// let observed = InlineIpld::wrap_with_link(DagCborCodec, &Sha2_256, ipld!([1, 2, 3]));
    /// assert_eq!(observed, ipld!({"/": {"data": ipld!([1,2, 3]), "link": cid}}));
    /// ```
    pub fn wrap_with_link<C: Codec, D: MultihashDigest<64>>(
        codec: C,
        digester: &D,
        ipld: Ipld,
    ) -> InlineIpld
    where
        Ipld: Encode<C>,
    {
        InlineIpld {
            ipld: ipld!({
              "/": {
                "data": ipld.clone(),
                "link": Ipld::Link(cid::new(&ipld, codec, digester, Version::V1).unwrap())
              }
            }),
        }
    }

    /// Tag some already-inlined [`Ipld`] to [`InlineIpld`]
    ///
    /// Use with caution: non-inlined [`Ipld`] can be maked as inlined with this function.
    ///
    /// This is used in place of [`from`][`From::from`] to highlight that no conversion or checks happen.
    /// If conversion is desired, use [`Self::wrap_inherit_link`].
    ///
    /// # Arguments
    ///
    /// * `ipld` - [`Ipld`] that is already correctly inlined
    ///
    /// # Examples
    ///
    /// ```
    /// # use inline_ipld::{cid, ipld::inlined::InlineIpld};
    /// # use std::str::FromStr;
    /// # use multihash::Code::Sha2_256;
    /// # use libipld::{
    /// #     cid::Version,
    /// #     cbor::DagCborCodec,
    /// #     ipld,
    /// #     Ipld,
    /// #     Cid
    /// # };
    /// #
    /// let ready = ipld!({"a": 1, "b": {"/": {"data": [1, 2, 3]}}});
    /// let observed = InlineIpld::attest(ready.clone());
    /// assert_eq!(observed, ready)
    /// ```
    pub fn attest(ipld: Ipld) -> Self {
        InlineIpld { ipld }
    }
}
