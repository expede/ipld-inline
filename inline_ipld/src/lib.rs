//! `inline_ipld` is an implementation of [`ucan-wg/ipld-inline-links`]
//!
//! [`ucan-wg/ipld-inline-links`]: https://github.com/ucan-wg/ipld-inline-links

#![warn(missing_docs)]

pub mod cid;
pub mod codec;
pub mod extractor;
pub mod inliner;
pub mod iterator;
pub mod store;

mod ipld;
pub use ipld::InlineIpld;

#[cfg(test)]
pub mod test_util;
