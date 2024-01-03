//! Inlining [`Ipld`][libipld::ipld::Ipld] from a content addressed store
mod at_least_once;
mod at_most_once;
mod traits;

pub use at_least_once::AtLeastOnce;
pub use at_most_once::AtMostOnce;
pub use traits::{Inliner, Stuck};
