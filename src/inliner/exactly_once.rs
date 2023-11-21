use super::quiet::Quiet;
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
        at_most_once.exactly_once.quiet
    }
}

impl<'a, S: Store + ?Sized> Iterator for AtMostOnce<'a, S> {
    type Item = Result<Ipld, Cid>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cid) = self.exactly_once.stuck_at {
            if !self.seen.contains(&cid) {
                return None;
            }
        }

        match self.exactly_once.next() {
            Some(Err(cid)) => {
                self.exactly_once.stuck_at = Some(cid);
                Some(Err(cid))
            }
            otherwise => otherwise,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ExactlyOnce<'a, S: Store + ?Sized> {
    quiet: Quiet<'a, S>,
    stuck_at: Option<Cid>,
}

impl<'a, S: Store + ?Sized> ExactlyOnce<'a, S> {
    pub fn new(ipld: Ipld, store: &'a S) -> Self {
        ExactlyOnce {
            quiet: Quiet::new(ipld, store),
            stuck_at: None,
        }
    }

    pub fn wants(&self) -> Option<Cid> {
        self.stuck_at
    }

    pub fn ignore(&mut self) -> Option<()> {
        self.stuck_at.map(|cid| {
            self.quiet.push(Ipld::Link(cid));
            self.stuck_at = None;
        })
    }

    // FIXME don't 100% love this
    pub fn resolve(&mut self, ipld: Ipld) -> Option<()> {
        self.stuck_at.map(|_| {
            self.quiet.push(ipld);
            self.stuck_at = None;
        })
    }
}

impl<'a, S: Store + ?Sized> From<Quiet<'a, S>> for ExactlyOnce<'a, S> {
    fn from(quiet: Quiet<'a, S>) -> Self {
        ExactlyOnce {
            quiet,
            stuck_at: None,
        }
    }
}

impl<'a, S: Store + ?Sized> From<ExactlyOnce<'a, S>> for Quiet<'a, S> {
    fn from(exactly_once: ExactlyOnce<'a, S>) -> Self {
        exactly_once.quiet
    }
}

impl<'a, S: Store + ?Sized> Iterator for ExactlyOnce<'a, S> {
    type Item = Result<Ipld, Cid>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stuck_at.is_some() {
            return None;
        }

        match self.quiet.next() {
            Some(Err(cid)) => {
                self.stuck_at = Some(cid);
                Some(Err(cid))
            }
            otherwise => otherwise,
        }
    }
}
