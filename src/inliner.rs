pub mod quiet;

use crate::iterator::post_order::PostOrderIpldIter;
use crate::store::traits::Store;
use libipld::{cid::Cid, ipld::Ipld};
use quiet::Quiet;
use std::{cell::RefCell, clone::Clone, ops::ControlFlow};

#[derive(Clone, Debug)]
pub struct Inliner<'a, S: Store> {
    inliner: Quiet<'a, S>,
    stuck_at: Option<Cid>,
}

impl<'a, S: Store> Inliner<'a, S> {
    pub fn new(ipld: Ipld, store: &'a S) -> Self {
        let inliner = Quiet {
            po_cell: RefCell::new(PostOrderIpldIter::from(ipld)),
            stack: vec![],
            store,
        };

        Inliner {
            inliner,
            stuck_at: None,
        }
    }

    pub fn wants(&self) -> Option<Cid> {
        self.stuck_at
    }

    pub fn next_until_stuck(&mut self) -> Option<Result<Ipld, Cid>> {
        let found = self.try_fold(None, |_, result| match result {
            Ok(_) => ControlFlow::Continue(Some(result)),
            Err(_) => ControlFlow::Break(Some(result)),
        });

        match found {
            ControlFlow::Break(opt_result) => opt_result,
            ControlFlow::Continue(opt_result) => opt_result,
        }
    }

    pub fn ignore(&mut self) -> Option<()> {
        self.stuck_at.map(|cid| {
            self.inliner.stack.push(Ipld::Link(cid));
            self.stuck_at = None;
        })
    }

    // FIXME don't 100% love this
    pub fn resolve(&mut self, ipld: Ipld) -> Option<()> {
        self.stuck_at.map(|_| {
            self.inliner.stack.push(ipld);
            self.stuck_at = None;
        })
    }
}

impl<'a, S: Store> From<Quiet<'a, S>> for Inliner<'a, S> {
    fn from(inliner: Quiet<'a, S>) -> Self {
        Inliner {
            inliner,
            stuck_at: None,
        }
    }
}

impl<'a, S: Store> From<Inliner<'a, S>> for Quiet<'a, S> {
    fn from(inliner: Inliner<'a, S>) -> Self {
        inliner.inliner
    }
}

impl<'a, S: Store> Iterator for Inliner<'a, S> {
    type Item = Result<Ipld, Cid>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stuck_at.is_some() {
            return None;
        }

        match self.inliner.next() {
            Some(Err(cid)) => {
                self.stuck_at = Some(cid);
                Some(Err(cid))
            }
            otherwise => otherwise,
        }
    }
}
