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
pub struct AtMostOnce<'a> {
    at_least_once: AtLeastOnce<'a>,
    seen: HashSet<Cid>,
}

impl<'a> AtMostOnce<'a> {
    /// Initialize a new [`AtMostOnce`] inliner
    pub fn new(ipld: &'a Ipld) -> Self {
        AtMostOnce {
            at_least_once: AtLeastOnce::new(ipld),
            seen: HashSet::new(),
        }
    }
}

impl<'a> From<&'a Ipld> for AtMostOnce<'a> {
    fn from(ipld: &'a Ipld) -> Self {
        AtMostOnce::new(ipld)
    }
}

impl<'a> From<AtLeastOnce<'a>> for AtMostOnce<'a> {
    fn from(at_least_once: AtLeastOnce<'a>) -> Self {
        AtMostOnce {
            at_least_once,
            seen: HashSet::new(),
        }
    }
}

impl<'a> From<AtMostOnce<'a>> for AtLeastOnce<'a> {
    fn from(at_most_once: AtMostOnce<'a>) -> Self {
        at_most_once.at_least_once
    }
}

impl<'a> Iterator for AtMostOnce<'a> {
    type Item = &'a Ipld;

    fn next(&mut self) -> Option<Self::Item> {
        self.at_least_once.next()
    }
}

impl<'a> Inliner<'a> for AtMostOnce<'a> {
    fn resolve(&mut self, ipld: Ipld) {
        self.at_least_once.resolve(ipld)
    }

    // FIXME by ref?
    fn run<S: Store + ?Sized>(self, store: &S) -> Option<Result<InlineIpld, Stuck<'a, Self>>> {
        match self.at_least_once.run(store)? {
            Ok(inline_ipld) => Some(Ok(inline_ipld)),
            Err(stuck) => {
                if self.seen.contains(stuck.needs()) {
                    let inliner: Self = (*stuck.ignore()).into();
                    inliner.run(store)
                } else {
                    None
                }
            }
        }
    }
}
