//! Helpers for building properly delimited and linked inline IPLD
//!
//! # Examples
//!
//! ```
//! # use ipld_inline::{cid, ipld::*};
//! # use multihash::Code::Sha2_256;
//! # use libipld::{
//! #     cid::Version,
//! #     cbor::DagCborCodec,
//! #     ipld,
//! #     Ipld
//! # };
//! #
//! let given_cid = cid::new(&ipld!([1,2,3]), DagCborCodec, &Sha2_256, Version::V1).unwrap();
//! let calculated_cid = cid::new(&ipld!([4, 5, 6]), DagCborCodec, &Sha2_256, Version::V1).unwrap();
//!
//! let observed = ipld!({
//!   "a": "foo",
//!   "b": inline_ipld_inherit(ipld!({
//!     "c": "bar",
//!     "d": inline_ipld(given_cid, ipld!([1, 2, 3]))
//!   })),
//!   "e": inline_ipld_link(DagCborCodec, &Sha2_256, ipld!([4, 5, 6]))
//! });
//!
//! assert_eq!(observed, ipld!(
//!   {
//!     "a": "foo",
//!     "b": {
//!       "/": {
//!         // "link": omited
//!         "data": {
//!           "c": "bar",
//!           "d": {"/": {"link": given_cid, "data": [1, 2, 3]}}
//!         }
//!       }
//!     },
//!     "e": {"/": {"link": calculated_cid, "data": [4, 5, 6]}
//!     }
//!   })
//! );
//! ```
use crate::cid;
use libipld::{
    cid::Version,
    codec::{Codec, Encode},
    ipld, Cid, Ipld,
};
use multihash::MultihashDigest;

/// Wrap some [`Ipld`] and manually associated [`Cid`] in an inline delimiter
///
/// To have the [`Cid`] calculated at runtime, use [`inline_ipld_link`].
/// [`inline_ipld`] *does not* check that the [`Cid`] actually corresponds to the associated [`Ipld`].
///
/// # Arguments
///
/// * `cid` - The [`Cid`] for the `"link"` field
/// * `ipld` - The [`Ipld`] to inline
///
/// ```
/// # use ipld_inline::{cid, ipld::inline_ipld};
/// # use libipld::{ipld, Ipld, Cid};
/// # use std::str::FromStr;
/// #
/// let cid: Cid = FromStr::from_str("bafyreihscx57i276zr5pgnioa5omevods6eseu5h4mllmow6csasju6eqi").unwrap();
/// assert_eq!(inline_ipld(cid, ipld!([1, 2, 3])), ipld!({
///   "/": {
///     "data": ipld!([1, 2, 3]),
///     "link": cid
///   }
/// }));
/// ```
pub fn inline_ipld(cid: Cid, ipld: Ipld) -> Ipld {
    ipld!({
      "/": {
        "data": ipld!(ipld),
        "link": Ipld::Link(cid)
      }
    })
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
/// # use ipld_inline::{cid, ipld::inline_ipld_inherit};
/// # use libipld::{ipld, Ipld};
/// #
/// let observed = inline_ipld_inherit(ipld!([1, 2, 3]));
/// assert_eq!(observed, ipld!({"/": {"data": ipld!([1, 2, 3])}}));
/// ```
pub fn inline_ipld_inherit(ipld: Ipld) -> Ipld {
    ipld!({"/": {"data": ipld!(ipld)}})
}

/// Create inline-delimited [`Ipld`], and automatically calculate the [`Cid`]
///
/// Unlike [`inline_ipld`], [`inline_ipld_link`] calculates a correct `"link"` based on
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
/// # use ipld_inline::{cid, ipld::inline_ipld_link};
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
/// let observed = inline_ipld_link(DagCborCodec, &Sha2_256, ipld!([1, 2, 3]));
/// assert_eq!(observed, ipld!({"/": {"data": ipld!([1,2, 3]), "link": cid}}));
/// ```
pub fn inline_ipld_link<C: Codec, D: MultihashDigest<64>>(
    codec: C,
    digester: &D,
    ipld: Ipld,
) -> Ipld
where
    Ipld: Encode<C>,
{
    ipld!({
      "/": {
        "data": ipld.clone(),
        "link": Ipld::Link(cid::new(&ipld, codec, digester, Version::V1).unwrap())
      }
    })
}
