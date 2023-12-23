//! Strategies for inlining [`Ipld`][libipld::ipld::Ipld]
//!
//! The primary interface for inlining is [`Inliner::run`]
pub mod at_most_once;
pub mod exactly_once;
pub mod naive;
pub mod traits;
