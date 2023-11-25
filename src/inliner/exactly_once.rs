use super::quiet::Quiet;
use crate::store::traits::Store;
use libipld::{cid::Cid, ipld::Ipld};

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
            None => None,
            Some(Err(cid)) => {
                self.stuck_at = Some(cid);
                Some(Err(cid))
            }
            Some(Ok(ipld)) => Some(Ok(ipld.clone())),
        }
    }
}
