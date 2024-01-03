//! [`Ipld`] marked as having passed [`Encode`]

use libipld::{
    codec::{Codec, Encode},
    error, Ipld,
};
use std::io::Write;

/// A wrapper around [`Ipld`] marked as being guaranteed to [`Encode::encode`] for some [`Codec`]
///
/// # Usage
///
/// * Use [`Ipld::to_encodable`] to construct a value of this type[^gdp].
/// * Use [`Into::into`] to extract the underlying [`Ipld`].
///
/// ```txt
/// ┌────────┐ to_encodable_as ┌─────────────────┐
/// │        ├─────────────────►                 │
/// │  Ipld  │                 │  EncodableIpld  │
/// │        ◄─────────────────┤                 │
/// └────────┘      into       └─────────────────┘
/// ```
///
/// [^gdp]: For more on this technique, see the [Ghosts of Departed Proofs](https://kataskeue.com/gdp.pdf) paper.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde-codec", derive(serde::Serialize))]
pub struct EncodableIpld<'a, C>
where
    C: Codec,
    Ipld: Encode<C>,
{
    // FIXME visibility
    pub(crate) ipld: &'a Ipld,
    pub(crate) codec: C,
}

impl<'a, C: Codec> EncodableIpld<'a, C>
where
    Ipld: Encode<C>,
{
    /// A wrapper around [`Encode::encode`] that is guaranteed to succeed
    ///
    /// # Examples
    ///
    /// ```
    /// # use inline_ipld::ipld::encodable::{EncodableAs, EncodableIpld};
    /// # use libipld::{ipld, codec::Codec};
    /// # use libipld_cbor::DagCborCodec;
    /// # use pretty_assertions::assert_eq;
    /// #
    /// let dag = ipld!([1, 2, 3]);
    /// let encodable = dag.to_encodable_as(DagCborCodec);
    /// let observed = encodable.guaranteed_encode();
    /// assert_eq!(observed, vec![0x83, 0x01, 0x02, 0x03]);
    /// ```
    pub fn guaranteed_encode(&self) -> Vec<u8> {
        self.codec
            .encode(self)
            .expect("should never fail if `to_encodable` is implemented correctly")
    }
}

impl<'a, C: Codec> From<EncodableIpld<'a, C>> for &'a Ipld
where
    Ipld: Encode<C>,
{
    fn from(enc_ipld: EncodableIpld<'a, C>) -> Self {
        enc_ipld.ipld
    }
}

impl<'a, C: Codec> Encode<C> for EncodableIpld<'a, C>
where
    Ipld: Encode<C>,
{
    fn encode<W: Write>(&self, c: C, w: &mut W) -> Result<(), error::Error> {
        self.ipld.encode(c, w)
    }
}
