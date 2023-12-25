//! Inlining [`Ipld`][libipld::ipld::Ipld] from a content addressed store
pub mod at_most_once;
pub mod exactly_once;
pub mod naive;
pub mod traits;
