//! Typesafe [`Ipld`][libipld::Ipld] encoding

mod encodable_ipld;
mod traits;

pub use encodable_ipld::EncodableIpld;
pub use traits::EncodableAs;
