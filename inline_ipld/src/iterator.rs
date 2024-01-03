//! Strategies for iterating over [`Ipld`][libipld::ipld::Ipld]
//!
//! [`libipld`]'s [`IpldIter`][libipld::ipld::IpldIter] is a pre-order traversal

mod post_order;

pub use post_order::{is_delimiter_next, PostOrderIpldIter};
