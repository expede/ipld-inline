//! `inline_ipld` is an implementation of [`ucan-wg/ipld-inline-links`]
//!
//! [`ucan-wg/ipld-inline-links`]: https://github.com/ucan-wg/ipld-inline-links
//!
//! # NOTE
//!
//! FIXME
//! Due to limitations in [`libipld`], the use of [`libipld_pb::DabPbCodec`],
//! [`libipld::RawCodec`], must be handled with care.

#![warn(missing_docs)]

pub mod cid;
pub mod extractor;
pub mod inliner;
pub mod ipld;
pub mod iterator;
pub mod store;

#[cfg(test)]
pub mod test_util;
