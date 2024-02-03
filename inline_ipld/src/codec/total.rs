//! The standard [`Codec`]s for [`Ipld`] that work on all inputs

use super::encodable::{EncodableAs, EncodableIpld};
use libipld::{
    codec::{Codec, Decode, Encode, References},
    error, Cid, Ipld,
};
use std::io::{Read, Seek, Write};

#[cfg(feature = "dag-cbor")]
use libipld_cbor::DagCborCodec;

#[cfg(feature = "dag-json")]
use libipld_json::DagJsonCodec;

/// The [`Codec`]s from [`libipld`] that are guaranteed to succeed encoding
///
/// These are [`Total`], as opposed to codecs that are only defined on some of (partial) [`Ipld`],
/// such as [`DagPbCodec`][libipld::pb::DagPbCodec] (and thus also [`IpldCodec`][libipld::IpldCodec]).
///
/// You are not limited to using these codecs. This is here for convience only,
/// and you can define your own instances of [`EncodableAs`].
///
/// The arms of the [`Total`] enum may be enabled/disabled with feature flags:
///
/// * `dag-cbor` - Enables [`DagCborCodec`]
/// * `dag-json` - Enables [`DagJsonCodec`]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Total {
    #[cfg(feature = "dag-cbor")]
    /// Entry for [`DagCborCodec`]
    DagCbor(DagCborCodec),

    #[cfg(feature = "dag-json")]
    /// Entry for [`DagJsonCodec`]
    DagJson(DagJsonCodec),
}

impl Codec for Total {}

impl TryFrom<u64> for Total {
    type Error = error::UnsupportedCodec;

    fn try_from(ccode: u64) -> core::result::Result<Self, Self::Error> {
        Ok(match ccode {
            #[cfg(feature = "dag-cbor")]
            0x71 => Total::DagCbor(DagCborCodec),

            #[cfg(feature = "dag-json")]
            0x0129 => Total::DagJson(DagJsonCodec),

            _ => return Err(error::UnsupportedCodec(ccode)),
        })
    }
}

impl From<Total> for u64 {
    fn from(mc: Total) -> Self {
        match mc {
            #[cfg(feature = "dag-cbor")]
            Total::DagCbor(_) => 0x71,

            #[cfg(feature = "dag-json")]
            Total::DagJson(_) => 0x0129,
        }
    }
}

#[cfg(feature = "dag-cbor")]
impl From<DagCborCodec> for Total {
    fn from(_: DagCborCodec) -> Self {
        Self::DagCbor(DagCborCodec)
    }
}

#[cfg(feature = "dag-cbor")]
impl From<Total> for DagCborCodec {
    fn from(_: Total) -> Self {
        Self
    }
}

#[cfg(feature = "dag-json")]
impl From<DagJsonCodec> for Total {
    fn from(_: DagJsonCodec) -> Self {
        Self::DagJson(DagJsonCodec)
    }
}

#[cfg(feature = "dag-json")]
impl From<Total> for DagJsonCodec {
    fn from(_: Total) -> Self {
        Self
    }
}

impl Encode<Total> for Ipld {
    fn encode<W: Write>(&self, c: Total, w: &mut W) -> Result<(), error::Error> {
        match c {
            #[cfg(feature = "dag-cbor")]
            Total::DagCbor(_) => self.encode(DagCborCodec, w)?,

            #[cfg(feature = "dag-json")]
            Total::DagJson(_) => self.encode(DagJsonCodec, w)?,
        };

        Ok(())
    }
}

impl Decode<Total> for Ipld {
    fn decode<R: Read + Seek>(c: Total, r: &mut R) -> Result<Self, error::Error> {
        Ok(match c {
            #[cfg(feature = "dag-cbor")]
            Total::DagCbor(_) => Self::decode(DagCborCodec, r)?,

            #[cfg(feature = "dag-json")]
            Total::DagJson(_) => Self::decode(DagJsonCodec, r)?,
        })
    }
}

impl References<Total> for Ipld {
    fn references<R: Read + Seek, E: Extend<Cid>>(
        c: Total,
        r: &mut R,
        set: &mut E,
    ) -> Result<(), error::Error> {
        match c {
            #[cfg(feature = "dag-cbor")]
            Total::DagCbor(_) => {
                <Self as References<DagCborCodec>>::references(DagCborCodec, r, set)?;
            }

            #[cfg(feature = "dag-json")]
            Total::DagJson(_) => {
                <Self as References<DagJsonCodec>>::references(DagJsonCodec, r, set)?;
            }
        };

        Ok(())
    }
}

impl EncodableAs<Total> for Ipld {
    fn encodable_as(&self, codec: Total) -> EncodableIpld<Total> {
        EncodableIpld { ipld: self, codec }
    }
}

#[cfg(test)]
use proptest::prelude::*;

#[cfg(test)]
impl Arbitrary for Total {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        prop_oneof![
            #[cfg(feature = "dag-cbor")]
            Just(Total::DagCbor(DagCborCodec)),
            #[cfg(feature = "dag-json")]
            Just(Total::DagJson(DagJsonCodec)),
        ]
        .boxed()
    }
}
