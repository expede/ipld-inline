use crate::iterator::post_order::PostOrderIpldIter;
use crate::store::traits::Store;
use libipld::{cid::Cid, ipld::Ipld};
use std::cell::RefCell;
use std::clone::Clone;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Inliner<'a, S: Store> {
    po_cell: RefCell<PostOrderIpldIter>,
    store: &'a S, // FIXME check clone performance
    stack: Vec<Ipld>,
}

impl<'a, S: Store + Clone> Inliner<'a, S> {
    pub fn new(ipld: Ipld, store: &'a S) -> Self {
        Inliner {
            po_cell: RefCell::new(PostOrderIpldIter::from(ipld)),
            stack: vec![],
            store,
        }
    }

    pub fn quiet_last(&mut self) -> Option<Ipld> {
        self.fold(None, |acc, result| match result {
            Ok(ipld) => Some(ipld),
            Err(stuck) => {
                stuck.ignore();
                acc
            }
        })
    }
}

impl<'a, S: Store + Clone> Iterator for Inliner<'a, S> {
    type Item = Result<Ipld, Stuck<'a, S>>;

    fn next(&mut self) -> Option<Self::Item> {
        for node in self.po_cell.get_mut() {
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
                            inliner: Box::new(self.clone()),
                        }));
                    }
                }

                Ipld::Map(btree) => {
                    let keys = btree.keys();
                    let vals = self.stack.split_off(self.stack.len() - keys.len());
                    let new_btree = keys.cloned().zip(vals).collect();
                    self.stack.push(Ipld::Map(new_btree));
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

#[derive(Clone, Debug)]
pub struct Stuck<'a, S: Store + Clone> {
    need: Cid,
    inliner: Box<Inliner<'a, S>>,
}

impl<'a, S: Store + Clone> Stuck<'a, S> {
    pub fn wants(&self) -> &Cid {
        &self.need
    }

    pub fn resolve(self, ipld: Ipld) -> Option<Inliner<'a, S>> {
        let mut il = *self.inliner;
        il.stack.push(ipld);
        Some(il)
    }

    pub fn ignore(self) -> Option<Inliner<'a, S>> {
        let cid = self.need;
        self.resolve(Ipld::Link(cid))
    }
}
