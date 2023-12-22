//! Inlining each subgraph at most once (deduplicated)
use super::{exactly_once::ExactlyOnce, quiet::Quiet};
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

impl<'a, S: Store> From<Quiet<'a, S>> for AtMostOnce<'a, S> {
    fn from(quiet: Quiet<'a, S>) -> Self {
        AtMostOnce {
            exactly_once: quiet.into(),
            seen: HashSet::new(),
        }
    }
}

impl<'a, S: Store> From<AtMostOnce<'a, S>> for Quiet<'a, S> {
    fn from(at_most_once: AtMostOnce<'a, S>) -> Self {
        at_most_once.exactly_once.into()
    }
}

impl<'a, S: Store + ?Sized> Iterator for AtMostOnce<'a, S> {
    type Item = Result<Ipld, Cid>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cid) = self.exactly_once.wants() {
            if !self.seen.contains(&cid) {
                return None;
            }
        }

        match self.exactly_once.next() {
            Some(Err(cid)) => {
                self.exactly_once.resolve(Ipld::Link(cid));
                Some(Err(cid))
            }
            otherwise => otherwise,
        }
    }
}
