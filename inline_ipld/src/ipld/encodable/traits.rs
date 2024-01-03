//! Make some [`Ipld`] statically guaranteed to [`encode`][Encode::encode]

// FIXME move to codecs?

use super::encodable_ipld::EncodableIpld;
use libipld::{
    codec::{Codec, Encode},
    Ipld,
};

#[cfg(feature = "dag-cbor")]
use libipld_cbor::DagCborCodec;

#[cfg(feature = "dag-json")]
use libipld_json::DagJsonCodec;

/// FIXME docs
///
///
/// This adds static guarantees to what would otherwise be runtime checks in [`encode`][libipld::codec::Encode::encode].
///
/// For example, without [`to_encodable`], the "stock" [`libipld`] behavior is as follows:
/// * [`RawCodec`][libipld::raw::RawCodec] fails on anything other than `Ipld::Bytes` or `Ipld::Bytes`
/// * [`DagPbCodec`][libipld_pb::DagPbCodec] fails to encode anything other than an `Ipld::Map` with a `"data"` key
///
/// This trait fixes that by only including codecs that are known to encode cleanly.
///
/// # Implementer's Guide
///
/// Implementation is straightforward: any type
pub trait EncodableAs<C>: Encode<C>
where
    C: Codec,
    Ipld: Encode<C>,
{
    /// Make some [`Ipld`] guaranteed to be [`encode`][libipld::codec::Encode::encode]able
    ///
    /// # Examples
    ///
    /// FIXME examples
    fn to_encodable_as(&self, codec: C) -> EncodableIpld<C>;
}

impl<'a, C: Codec> EncodableAs<C> for EncodableIpld<'a, C>
where
    Ipld: EncodableAs<C>,
{
    fn to_encodable_as(&self, _codec: C) -> EncodableIpld<'a, C> {
        *self
    }
}

#[cfg(feature = "dag-json")]
impl EncodableAs<DagJsonCodec> for Ipld {
    fn to_encodable_as(&self, codec: DagJsonCodec) -> EncodableIpld<DagJsonCodec> {
        EncodableIpld { ipld: self, codec }
    }
}

#[cfg(feature = "dag-cbor")]
impl EncodableAs<DagCborCodec> for Ipld {
    fn to_encodable_as(&self, codec: DagCborCodec) -> EncodableIpld<DagCborCodec> {
        EncodableIpld { ipld: self, codec }
    }
}
