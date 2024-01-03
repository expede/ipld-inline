// FIXME rename to encodable codec... or something?

use crate::ipld::encodable::{EncodableAs, EncodableIpld};
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
/// You are not limited to using these codecs. This is here for convience only,
/// and you can define your own instances of [`EncodableAs`][crate::ipld::encodable::EncodableAs].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SafeCodec {
    #[cfg(feature = "dag-cbor")]
    /// Entry for [`DagCborCodec`]
    DagCbor(DagCborCodec),

    #[cfg(feature = "dag-json")]
    /// Entry for [`DagJsonCodec`]
    DagJson(DagJsonCodec),
}

impl Codec for SafeCodec {}

impl TryFrom<u64> for SafeCodec {
    type Error = error::UnsupportedCodec;

    fn try_from(ccode: u64) -> core::result::Result<Self, Self::Error> {
        Ok(match ccode {
            #[cfg(feature = "dag-cbor")]
            0x71 => SafeCodec::DagCbor(DagCborCodec),

            #[cfg(feature = "dag-json")]
            0x0129 => SafeCodec::DagJson(DagJsonCodec),

            _ => return Err(error::UnsupportedCodec(ccode)),
        })
    }
}

impl From<SafeCodec> for u64 {
    fn from(mc: SafeCodec) -> Self {
        match mc {
            #[cfg(feature = "dag-cbor")]
            SafeCodec::DagCbor(_) => 0x71,

            #[cfg(feature = "dag-json")]
            SafeCodec::DagJson(_) => 0x0129,
        }
    }
}

#[cfg(feature = "dag-cbor")]
impl From<DagCborCodec> for SafeCodec {
    fn from(_: DagCborCodec) -> Self {
        Self::DagCbor(DagCborCodec)
    }
}

#[cfg(feature = "dag-cbor")]
impl From<SafeCodec> for DagCborCodec {
    fn from(_: SafeCodec) -> Self {
        Self
    }
}

#[cfg(feature = "dag-json")]
impl From<DagJsonCodec> for SafeCodec {
    fn from(_: DagJsonCodec) -> Self {
        Self::DagJson(DagJsonCodec)
    }
}

#[cfg(feature = "dag-json")]
impl From<SafeCodec> for DagJsonCodec {
    fn from(_: SafeCodec) -> Self {
        Self
    }
}

impl Encode<SafeCodec> for Ipld {
    fn encode<W: Write>(&self, c: SafeCodec, w: &mut W) -> Result<(), error::Error> {
        match c {
            #[cfg(feature = "dag-cbor")]
            SafeCodec::DagCbor(_) => self.encode(DagCborCodec, w)?,

            #[cfg(feature = "dag-json")]
            SafeCodec::DagJson(_) => self.encode(DagJsonCodec, w)?,
        };

        Ok(())
    }
}

impl Decode<SafeCodec> for Ipld {
    fn decode<R: Read + Seek>(c: SafeCodec, r: &mut R) -> Result<Self, error::Error> {
        Ok(match c {
            #[cfg(feature = "dag-cbor")]
            SafeCodec::DagCbor(_) => Self::decode(DagCborCodec, r)?,

            #[cfg(feature = "dag-json")]
            SafeCodec::DagJson(_) => Self::decode(DagJsonCodec, r)?,
        })
    }
}

impl References<SafeCodec> for Ipld {
    fn references<R: Read + Seek, E: Extend<Cid>>(
        c: SafeCodec,
        r: &mut R,
        set: &mut E,
    ) -> Result<(), error::Error> {
        match c {
            #[cfg(feature = "dag-cbor")]
            SafeCodec::DagCbor(_) => {
                <Self as References<DagCborCodec>>::references(DagCborCodec, r, set)?;
            }

            #[cfg(feature = "dag-json")]
            SafeCodec::DagJson(_) => {
                <Self as References<DagJsonCodec>>::references(DagJsonCodec, r, set)?;
            }
        };

        Ok(())
    }
}

impl EncodableAs<SafeCodec> for Ipld {
    fn to_encodable_as(&self, codec: SafeCodec) -> EncodableIpld<SafeCodec> {
        EncodableIpld { ipld: self, codec }
    }
}

#[cfg(test)]
use proptest::prelude::*;

#[cfg(test)]
impl Arbitrary for SafeCodec {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        prop_oneof![
            #[cfg(feature = "dag-cbor")]
            Just(SafeCodec::DagCbor(DagCborCodec)),
            #[cfg(feature = "dag-json")]
            Just(SafeCodec::DagJson(DagJsonCodec)),
        ]
        .boxed()
    }
}
