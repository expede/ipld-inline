use super::{exactly_once::ExactlyOnce, quiet::Quiet};
use crate::store::traits::Store;
use libipld::{cid::Cid, ipld::Ipld};
use std::{clone::Clone, collections::HashSet};

#[derive(Clone, Debug)]
pub struct AtMostOnce<'a, S: Store + ?Sized> {
    exactly_once: ExactlyOnce<'a, S>,
    seen: HashSet<Cid>,
}

impl<'a, S: Store + ?Sized> AtMostOnce<'a, S> {
    pub fn new(ipld: Ipld, store: &'a S) -> Self {
        let exactly_once = ExactlyOnce::new(ipld, store);
        AtMostOnce {
            exactly_once,
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
