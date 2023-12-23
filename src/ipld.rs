//! Helpers for building properly delimited and linked inline IPLD
use libipld::{ipld, Ipld};

pub const DELIMIT_INLINE: &'static str = "/";
pub const DATA_TAG: &'static str = "data";
pub const LINK_TAG: &'static str = "link";

/// A helper for creating properly delimited inline IPLD
///
/// `inline_ipld!` automatically wraps the enclosed [`Ipld`] in the correct delimiters.
/// It has three arities:
///
/// ## Unary
///
/// ```no_run
/// # use ipld_inline::{cid, inline_ipld};
/// # use libipld::{ipld, Ipld};
/// #
/// # let ipld = Ipld::Null;
/// #
/// inline_ipld!(ipld);
/// ```
///
/// The unary variant omits the `"link"` field. Per the [spec], this causes the extracted
/// IPLD link to inherit the coedec and CID encoding of its parent.
///
/// [spec]: https://github.com/ucan-wg/ipld-inline-links
///
/// ## Binary
///
/// ```no_run
/// # use ipld_inline::{cid, inline_ipld};
/// # use libipld::{ipld, Ipld, Cid};
/// # use std::str::FromStr;
/// #
/// # let ipld = ipld!("");
/// # let cid: Cid = FromStr::from_str("bafyreihscx57i276zr5pgnioa5omevods6eseu5h4mllmow6csasju6eqi").unwrap();
/// #
/// inline_ipld!(cid, ipld);
/// ```
///
/// The binary variant accepts an explicit [`Cid`][libipld::cid::Cid] parameter.
/// This *does not* check that the [`Cid`][libipld::cid::Cid] actually corresponds to the associated [`Ipld`].
///
/// ## Ternary
///
/// ```no_run
/// # use ipld_inline::{cid, inline_ipld};
/// # use multihash::Code::Sha2_256;
/// # use libipld::{
/// #     cid::Version,
/// #     cbor::DagCborCodec,
/// #     ipld,
/// #     Ipld
/// # };
/// #
/// # let digester = DagCborCodec;
/// # let codec = Sha2_256;
/// # let ipld = Ipld::Null;
/// #
/// inline_ipld!(digester, codec, ipld);
/// ```
///
/// The ternary variant calculates the correct `"link"` (at runtime) based on the configuration passed in.
///
/// # Examples
///
/// The following example includes all arities of `inline_ipld!`.
///
/// ```no_run
/// # use ipld_inline::{cid, inline_ipld};
/// # use multihash::Code::Sha2_256;
/// # use libipld::{
/// #     cid::Version,
/// #     cbor::DagCborCodec,
/// #     ipld,
/// #     Ipld
/// # };
/// #
/// let given_cid = cid::new(&ipld!([1,2,3]), DagCborCodec, &Sha2_256, Version::V1).unwrap();
/// let calculated_cid = cid::new(&ipld!([4, 5, 6]), DagCborCodec, &Sha2_256, Version::V1).unwrap();
///
/// let observed = ipld!({
///   "a": "foo",
///   "b": inline_ipld!({ // Unary
///     "c": "bar",
///     "d": inline_ipld!(given_cid, [1, 2, 3]) // Binary
///   }),
///   "e": inline_ipld!(DagCborCodec, Sha2_256, [4, 5, 6]) // Ternary
/// });
///
/// assert_eq!(observed, ipld!(
///   {
///     "a": "foo",
///     "b": {
///       "/": {
///         // Unary omits the `"link"` field
///         // "link": not here,
///         "data": {
///           "c": "bar",
///           // Binary includes the given `"link"`
///           //                  vvvvvvvvv
///           "d": {"/": {"link": given_cid, "data": [1, 2, 3]}}
///         }
///       }
///     },
///     // Ternary uses the calculated `"link"`
///     //                  vvvvvvvvvvvvvv
///     "e": {"/": {"link": calculated_cid, "data": [4, 5, 6]}
///     }
///   })
/// );
/// ```
#[macro_export]
macro_rules! inline_ipld {
   ($ipld: tt) => {
     ipld!({"/": {"data": ipld!($ipld)}})
   };

   ($cid: tt, $ipld: tt) => {
     ipld!({
       "/": {
         "data": ipld!($ipld),
         "link": Ipld::Link($cid)
       }
     })
   };

   ($digester: tt, $codec: tt, $ipld: tt) => {
     ipld!({
       "/": {
         "data": ipld!($ipld),
         "link": Ipld::Link(cid::new(&ipld!($ipld), $digester, &$codec, Version::V1).unwrap())
       }
     })
   };
}
