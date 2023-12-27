//! Inline each subgraph at _most_ once (with deduplication)
use super::{
    at_least_once::AtLeastOnce,
    traits::{Inliner, Stuck},
};
use crate::{ipld::inlined::InlineIpld, store::traits::Store};
use libipld::{cid::Cid, ipld::Ipld};
use std::collections::HashSet;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// [`Ipld`] inliner that only inlines a [`Cid`] at most once, if avalaible
///
/// This behaves as an "exactly once" if all subgraphs are available
/// (e.g. [`Stuck::ignore`] is never called).
///
/// In general, you should prefer the use of the [`Inliner`] interface, over [`Iterator`].
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct AtMostOnce {
    at_least_once: AtLeastOnce,
    seen: HashSet<Cid>,
}

impl AtMostOnce {
    /// Initialize a new [`AtMostOnce`] inliner
    pub fn new(ipld: Ipld) -> Self {
        AtMostOnce {
            at_least_once: AtLeastOnce::new(ipld),
            seen: HashSet::new(),
        }
    }
}

impl From<Ipld> for AtMostOnce {
    fn from(ipld: Ipld) -> Self {
        AtMostOnce::new(ipld)
    }
}

impl From<AtLeastOnce> for AtMostOnce {
    fn from(at_least_once: AtLeastOnce) -> Self {
        AtMostOnce {
            at_least_once,
            seen: HashSet::new(),
        }
    }
}

impl From<AtMostOnce> for AtLeastOnce {
    fn from(at_most_once: AtMostOnce) -> Self {
        at_most_once.at_least_once
    }
}

impl Iterator for AtMostOnce {
    type Item = Ipld;

    fn next(&mut self) -> Option<Self::Item> {
        self.at_least_once.next()
    }
}

impl<'a> Inliner<'a> for AtMostOnce {
    fn resolve(&mut self, ipld: Ipld) {
        self.at_least_once.resolve(ipld)
    }

    fn run<S: Store + ?Sized>(
        &'a mut self,
        store: &S,
    ) -> Option<Result<InlineIpld, Stuck<'a, Self>>> {
        let result = self.at_least_once.run(store)?;
        match result {
            Ok(inline) => Some(Ok(inline)),
            Err(Stuck { needs, .. }) => {
                if self.seen.contains(&needs) {
                    self.at_least_once.resolve(Ipld::Link(needs));
                    let inline = InlineIpld::attest(Ipld::Link(needs));
                    Some(Ok(inline))
                } else {
                    None
                }
            }
        }
    }
}
