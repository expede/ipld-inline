//! Naive inlining
//!
//! This inlining strategy tries its best, but:
//! - Doesn't attempt to deduplicate DAGs
//! - Doesn't stop if a [`Cid`] is not available in the attached [`Store`]

use crate::{
    inliner::traits::Inliner, iterator::post_order::PostOrderIpldIter, store::traits::Store,
};
use libipld::{cid::Cid, ipld::Ipld};
use std::{clone::Clone, collections::BTreeMap};

#[cfg(feature = "serde")]
use serde::Serialize;

/// Inline directly, without deduplication or stopping at missing nodes
///
/// More sophisticated inlining strategies are available in the [`Inliner`][inline_ipld::inliner] module
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Naive<'a, S: Store + ?Sized> {
    po: PostOrderIpldIter,
    stack: Vec<Ipld>,
    pub(crate) store: &'a mut S,
}

impl<'a, S: Store + ?Sized> Naive<'a, S> {
    /// Initialize a new [`Naive`] inliner
    ///
    /// # Arguments
    ///
    /// - `ipld` - The [`Ipld`] to inline
    /// - `store` - The content addressed [`Store`] to draw graphs from
    pub fn new(ipld: Ipld, store: &'a mut S) -> Self {
        Naive {
            po: PostOrderIpldIter::from(ipld),
            stack: vec![],
            store,
        }
    }

    pub(super) fn push(&mut self, ipld: Ipld) {
        self.stack.push(ipld);
    }
}

impl<'a, S: Store + ?Sized> Inliner for Naive<'a, S> {
    fn store(&mut self, cid: &Cid, ipld: &Ipld) {
        self.store
    }
}

impl<'a, S: Store + ?Sized> Iterator for Naive<'a, S> {
    type Item = Result<Ipld, Cid>;

    fn next(&mut self) -> Option<Self::Item> {
        for node in &mut self.po {
            match node {
                Ipld::Link(cid) => {
                    if let Ok(ipld) = self.store.get(&cid) {
                        let mut inner = BTreeMap::new();
                        inner.insert("link".to_string(), Ipld::Link(cid));
                        inner.insert("data".to_string(), ipld.clone());

                        let mut outer = BTreeMap::new();
                        outer.insert("/".to_string(), Ipld::Map(inner));

                        self.stack.push(Ipld::Map(outer));
                    } else {
                        return Some(Err(cid));
                    }
                }

                Ipld::Map(btree) => {
                    let keys = btree.keys();
                    let vals: Vec<Ipld> = self.stack.split_off(self.stack.len() - keys.len());
                    let new_btree = keys.cloned().zip(vals).collect();
                    self.stack.push(Ipld::Map(new_btree));
                }

                Ipld::List(vec) => {
                    let new_vec = self.stack.split_off(self.stack.len() - vec.len());
                    self.stack.push(Ipld::List(new_vec));
                }

                node => {
                    self.stack.push(node);
                }
            }
        }

        self.stack.pop().map(Ok)
    }
}
