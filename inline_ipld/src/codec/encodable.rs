//! Codecs that are guaranteed to work for all [`Ipld`][libipld::Ipld] inputs

mod ipld;
mod traits;

pub use ipld::EncodableIpld;
pub use traits::EncodableAs;
