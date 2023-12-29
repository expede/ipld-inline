//! Make some [`Ipld`] statically guaranteed to [`encode`][Encode::encode]

use super::encodable_ipld::EncodableIpld;
use libipld::{
    codec::{Codec, Encode},
    Ipld,
};

#[cfg(feature = "dag-cbor")]
use libipld_cbor::DagCborCodec;

#[cfg(feature = "dag-json")]
use libipld_json::DagJsonCodec;

/// FIXME
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
    /// This adds static guarantees to what would otherwise be runtime checks in [`encode`][libipld::codec::Encode::encode].
    ///
    /// For example, without [`to_encodable`], the "stock" [`libipld`] behavior is as follows:
    /// * [`RawCodec`][libipld::raw::RawCodec] fails on anything other than `Ipld::Bytes` or `Ipld::Bytes`
    /// * [`DagPbCodec`][libipld_pb::DagPbCodec] fails to encode anything other than an `Ipld::Map` with a `"data"` key
    ///
    /// # Examples
    ///
    /// FIXME
    fn to_encodable_as<'a>(&'a self, codec: C) -> EncodableIpld<'a, C>;
}

impl<'a, C: Codec> EncodableAs<C> for EncodableIpld<'a, C>
where
    Ipld: EncodableAs<C>,
{
    fn to_encodable_as(&self, _codec: C) -> EncodableIpld<'a, C> {
        todo!()
        // self.clone()
    }
}

#[cfg(feature = "dag-json")]
impl EncodableAs<DagJsonCodec> for Ipld {
    fn to_encodable_as<'a>(&'a self, codec: DagJsonCodec) -> EncodableIpld<'a, DagJsonCodec> {
        EncodableIpld { ipld: self, codec }
    }
}

#[cfg(feature = "dag-cbor")]
impl EncodableAs<DagCborCodec> for Ipld {
    fn to_encodable_as<'a>(&'a self, codec: DagCborCodec) -> EncodableIpld<'a, DagCborCodec> {
        EncodableIpld { ipld: self, codec }
    }
}

// // // FIXME what a PITA
// // // #[cfg(feature = "dag-pb")]
// // // impl Encodable<Ipld> for DagPbCodec {
// // //     fn to_encodable(&self, ipld: Ipld) -> Encodable<Self> {
// // //         let encodable = match (ipld.get("Data"), ipld.get("Links")) {
// // //             (Ok(_), Ok(_)) => ipld,
// // //             _ => {
// // //                 let refs = ipld.try_into();
// // //                 let mut btree: BTreeMap<String, Ipld> = Default::default();
// // //                 btree.insert("Data".into(), Ipld::Bytes(ipld)); // FIXME MUST be `Bytes`
// // //                 btree.insert("Links".into(), Ipld::List(refs));
// // //                 Ipld::Map(btree)
// // //             }
// // //         };
// // //
// // //         Encodable {
// // //             encodable,
// // //             phantom: PhantomData,
// // //         }
// // //     }
// // // }
