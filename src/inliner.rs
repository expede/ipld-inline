use crate::iterator::post_order::PostOrderIpldIter;
use crate::store::Store;
use libipld::{cid::Cid, ipld::Ipld};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

// FIXME do versions that 1. always inline, and 2. only once inlines

#[derive(Clone, Debug)]
pub struct Inliner<S: Store> {
    po_cell: RefCell<PostOrderIpldIter>,
    store: Rc<S>,
    stack: Vec<Ipld>,
}

impl<S: Store + Clone> Inliner<S> {
    pub fn new(ipld: Ipld, store: Rc<S>) -> Self {
        Inliner {
            po_cell: RefCell::new(PostOrderIpldIter::from(ipld)),
            stack: vec![],
            store,
        }
    }

    pub fn do_your_best(&mut self) -> Option<Ipld> {
        self.fold(None, |acc, result| match result {
            Ok(ipld) => Some(ipld),
            Err(stuck) => {
                stuck.ignore();
                acc
            }
        })
    }
}

impl<S: Store + Clone> Iterator for Inliner<S> {
    type Item = Result<Ipld, Stuck<S>>;

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
                            inliner: Rc::new(self.clone()),
                        }));
                    }
                }

                Ipld::Map(btree) => {
                    let keys = btree.keys();
                    let vals = self.stack.split_off(self.stack.len() - keys.len());
                    let new_btree = keys.cloned().zip(vals).collect();
                    self.stack = vec![Ipld::Map(new_btree)];
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
pub struct Stuck<S: Store> {
    need: Cid,
    inliner: Rc<Inliner<S>>,
}

impl<S: Store> Stuck<S> {
    pub fn wants(&self) -> &Cid {
        &self.need
    }

    pub fn resolve(self, ipld: Ipld) -> Option<Inliner<S>> {
        if let Ok(mut il) = Rc::try_unwrap(self.inliner) {
            il.stack.push(ipld);
            Some(il)
        } else {
            None
        }
    }

    pub fn ignore(self) -> Option<Inliner<S>> {
        let cid = self.need;
        self.resolve(Ipld::Link(cid))
    }
}
