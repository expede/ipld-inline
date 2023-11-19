use crate::iterator::post_order::PostOrderIpldIter;
use crate::store::Store;
use libipld::{cid::Cid, ipld::Ipld};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Inliner<S: Store> {
    iterator: PostOrderIpldIter,
    stack: Vec<Ipld>,
    store: S,
}

impl<S: Store> Inliner<S> {
    pub fn new(ipld: Ipld, store: S) -> Self {
        Inliner {
            iterator: PostOrderIpldIter::from(ipld),
            stack: vec![],
            store,
        }
    }
}

impl<S: Store> Iterator for Inliner<S> {
    type Item = Result<Ipld, Stuck<S>>;

    fn next(&mut self) -> Option<Self::Item> {
        for node in self.iterator.clone() {
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
                        return Some(Err(Stuck {
                            need: cid,
                            inliner: self.clone(),
                        }));
                    }
                }

                Ipld::Map(btree) => {
                    let keys = btree.keys();
                    let vals = self.stack.split_off(self.stack.len() - keys.len());
                    let new_btree = keys.cloned().zip(vals).collect();
                    self.stack = vec![Ipld::Map(new_btree)]
                }

                Ipld::List(vec) => {
                    let new_vec = self.stack.split_off(self.stack.len() - vec.len());
                    self.stack.push(Ipld::List(new_vec));
                }

                node => {
                    self.stack.push(node.clone());
                }
            }
        }

        let root = self
            .stack
            .pop()
            .expect("should have exactly one item on the stack");

        Some(Ok(root))
    }
}

#[derive(Debug)]
pub struct Stuck<S: Store> {
    need: Cid,
    inliner: Inliner<S>,
}

impl<S: Store> Stuck<S> {
    pub fn wants(&self) -> &Cid {
        &self.need
    }

    pub fn ignore(self) -> Inliner<S> {
        self.inliner
    }

    pub fn resolve(&mut self, ipld: Ipld) -> &Inliner<S> {
        self.inliner.iterator.inbound.push(ipld);
        &self.inliner
    }
}
