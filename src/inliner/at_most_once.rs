//! Inlining each subgraph at most once (deduplicated)
use super::{exactly_once::ExactlyOnce, naive::Naive, traits::*};
use crate::store::traits::Store;
use libipld::{cid::Cid, ipld::Ipld};
use std::collections::HashSet;

/// [`Ipld`] inliner that only inlines a [`Cid`] once, if avalaible
///
/// If a subgraph isn't available by the required [`Cid`], it's merely skipped
#[derive(Debug)]
pub struct AtMostOnce<'a, S: Store + ?Sized> {
    exactly_once: ExactlyOnce<'a, S>,
    seen: HashSet<Cid>,
}

impl<'a, S: Store + ?Sized> AtMostOnce<'a, S> {
    /// Initialize a new [`AtMostOnce`] inliner
    ///
    /// # Arguments
    ///
    /// - `ipld` - The [`Ipld`] to inline
    /// - `store` - The content addressed [`Store`] to draw graphs from
    pub fn new(ipld: Ipld, store: &'a mut S) -> Self {
        AtMostOnce {
            exactly_once: ExactlyOnce::new(ipld, store),
            seen: HashSet::new(),
        }
    }
}

impl<'a, S: Store> From<ExactlyOnce<'a, S>> for AtMostOnce<'a, S> {
    fn from(exactly_once: ExactlyOnce<'a, S>) -> Self {
        AtMostOnce {
            exactly_once,
            seen: HashSet::new(),
        }
    }
}

impl<'a, S: Store> From<AtMostOnce<'a, S>> for ExactlyOnce<'a, S> {
    fn from(at_most_once: AtMostOnce<'a, S>) -> Self {
        at_most_once.exactly_once
    }
}

impl<'a, S: Store> From<Naive<'a, S>> for AtMostOnce<'a, S> {
    fn from(naive: Naive<'a, S>) -> Self {
        AtMostOnce {
            exactly_once: naive.into(),
            seen: HashSet::new(),
        }
    }
}

impl<'a, S: Store> From<AtMostOnce<'a, S>> for Naive<'a, S> {
    fn from(at_most_once: AtMostOnce<'a, S>) -> Self {
        at_most_once.exactly_once.into()
    }
}

impl<'a, S: Store + ?Sized> Iterator for AtMostOnce<'a, S> {
    type Item = Result<Ipld, Cid>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.exactly_once.next()? {
            Err(needs) => {
                if self.seen.contains(&needs) {
                    self.exactly_once.interject(&Ipld::Link(needs));
                    Some(Ok(Ipld::Link(needs)))
                } else {
                    None
                }
            }
            good => Some(good),
        }
    }
}

impl<'a, S: Store + ?Sized> Inliner<'a> for AtMostOnce<'a, S> {
    fn store(&mut self, cid: &Cid, ipld: &Ipld) {
        self.exactly_once.store(cid, ipld);
    }

    fn interject(&mut self, ipld: &Ipld) {
        self.exactly_once.interject(ipld)
    }
}
