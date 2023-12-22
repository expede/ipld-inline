//! Naive inlining
//!
//! This inlining strategy tries its best, but:
//! - Doesn't attempt to deduplicate DAGs
//! - Doesn't stop if a [`Cid`] is not available in the attached [`Store`]

// FIXME rename this module Naive
use crate::iterator::post_order::PostOrderIpldIter;
use crate::store::traits::Store;
use libipld::{cid::Cid, ipld::Ipld};
use std::{clone::Clone, collections::BTreeMap};

#[derive(Debug)]
pub struct Quiet<'a, S: Store + ?Sized> {
    po: PostOrderIpldIter,
    stack: Vec<Ipld>,
    pub(crate) store: &'a mut S,
}

impl<'a, S: Store + ?Sized> Quiet<'a, S> {
    pub fn new(ipld: Ipld, store: &'a mut S) -> Self {
        Quiet {
            po: PostOrderIpldIter::from(ipld),
            stack: vec![],
            store,
        }
    }

    pub(super) fn push(&mut self, ipld: Ipld) {
        self.stack.push(ipld);
    }
}

impl<'a, S: Store + ?Sized> Iterator for Quiet<'a, S> {
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
