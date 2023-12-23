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
/// ```ignore
/// inline_ipld!(ipld: Ipld)
/// ```
///
/// The unary variant omits the `"link"` field. Per the [spec], this causes the extracted
/// IPLD link to inherit the coedec and CID encoding of its parent.
///
/// [spec]: https://github.com/ucan-wg/ipld-inline-links
///
/// ## Binary
///
/// ```ignore
/// inline_ipld!(cid: Cid, ipld: Ipld)
/// ```
///
/// The binary variant accepts an explicit [`Cid`][libipld::cid::Cid] parameter.
/// This *does not* check that the [`Cid`][libipld::cid::Cid] actually corresponds to the associated [`Ipld`].
///
/// ## Ternary
///
/// ```ignore
/// inline_ipld!(digester: MultihashDigest<64>, codec: Codec, ipld: Ipld)
/// ```
///
/// The ternary variant calculates the correct `"link"` (at runtime) based on the configuration passed in.
///
/// # Examples
///
/// The following example includes all arities of `inline_ipld!`.
///
/// ```
/// # use ipld_inline::{ipld::*, inline_ipld};
/// # use ipld_inline::cid;
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
    ipld!({
       DELIMIT_INLINE: {
         DATA_TAG: ipld!($ipld),
       }
     })
   };

   ($cid: tt, $ipld: tt) => {
     ipld!({
       DELIMIT_INLINE: {
         DATA_TAG: ipld!($ipld),
         LINK_TAG: Ipld::Link($cid)
       }
     })
   };

   ($digester: tt, $codec: tt, $ipld: tt) => {
     ipld!({
       DELIMIT_INLINE: {
         DATA_TAG: ipld!($ipld),
         LINK_TAG: Ipld::Link(cid::new(&ipld!($ipld), $digester, &$codec, Version::V1).unwrap())
       }
     })
   };
}
