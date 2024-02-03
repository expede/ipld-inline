//! Signal InlineIpld IPLD formatted [`Ipld`]

use libipld::{ipld, Cid, Ipld};

/// InlineIpld IPLD formatted [`Ipld`]
///
/// This is helpful to indictate that some form of inlining has already been performed.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde-codec", derive(serde::Deserialize, serde::Serialize))]
pub struct InlineIpld {
    ipld: Ipld,
    cid: Option<Cid>,
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

impl TryFrom<InlineIpld> for Cid {
    type Error = ();

    fn try_from(inline: InlineIpld) -> Result<Cid, ()> {
        inline.cid.ok_or(())
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
    /// Note that [`new`][Self::new] *does not* check that the [`Cid`] actually corresponds to the associated [`Ipld`].
    ///
    /// # Arguments
    ///
    /// * `cid` - The [`Cid`] for the `"link"` field
    /// * `ipld` - The [`Ipld`] to inline
    ///
    /// ```
    /// # use inline_ipld::{cid, InlineIpld};
    /// # use libipld::{ipld, Ipld, Cid};
    /// # use std::str::FromStr;
    /// #
    /// let cid: Cid = FromStr::from_str("bafyreihscx57i276zr5pgnioa5omevods6eseu5h4mllmow6csasju6eqi").unwrap();
    /// assert_eq!(InlineIpld::new(cid, ipld!([1, 2, 3])), ipld!({
    ///   "/": {
    ///     "data": ipld!([1, 2, 3]),
    ///     "link": cid
    ///   }
    /// }));
    /// ```
    /// FIXME test ipld! macro with InlineIpld to see if it wraps automagically
    pub fn new(cid: Cid, ipld: Ipld) -> Self {
        InlineIpld {
            cid: Some(cid),
            ipld: ipld!({
              "/": {
                "link": Ipld::Link(cid),
                "data": ipld
              }
            }),
        }
    }

    /// Create inline-delimited [`Ipld`], but omit the `"link"` field
    ///
    /// [`new_inherit_link`][Self::new_inherit_link] wraps the enclosed [`Ipld`] in the correct delimiters.
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
    /// # use inline_ipld::{cid, InlineIpld};
    /// # use libipld::{ipld, Ipld};
    /// #
    /// let observed = InlineIpld::new_inherit_link(ipld!([1, 2, 3]));
    /// assert_eq!(observed, ipld!({"/": {"data": ipld!([1, 2, 3])}}));
    /// ```
    pub fn new_inherit_link(ipld: Ipld) -> Self {
        InlineIpld {
            cid: None,
            ipld: ipld!({"/": {"data": ipld!(ipld)}}),
        }
    }

    /// Tag some already-inlined [`Ipld`] to [`InlineIpld`]
    ///
    /// Use with caution: non-inlined [`Ipld`] can be maked as inlined with this function.
    ///
    /// This is used in place of [`from`][`From::from`] to highlight that no conversion or checks happen.
    /// If conversion is desired, use [`Self::new_inherit_link`].
    ///
    /// # Arguments
    ///
    /// * `ipld` - [`Ipld`] that is already correctly inlined
    ///
    /// # Examples
    ///
    /// ```
    /// # use inline_ipld::{cid, InlineIpld};
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
        InlineIpld { ipld, cid: None }
    }

    /// Retrieve the (optional) [`Cid`]
    ///
    /// # Examples
    ///
    /// ```
    /// # use inline_ipld::{cid, InlineIpld};
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
    /// let ipld = ipld!({"a": 1, "b": {"/": {"data": [1, 2, 3]}}});
    /// let cid = cid::new(&ipld, DagCborCodec, &Sha2_256, Version::V1);
    /// let inlined = InlineIpld::new(cid, ipld);
    /// assert_eq!(inlined.cid(), Some(cid));
    /// ```
    pub fn cid(&self) -> Option<Cid> {
        self.cid
    }
}
