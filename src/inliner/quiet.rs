use crate::iterator::post_order::PostOrderIpldIter;
use crate::store::traits::Store;
use libipld::{cid::Cid, ipld::Ipld};
use std::{cell::RefCell, clone::Clone, collections::BTreeMap};

#[derive(Clone, Debug)]
pub struct Quiet<'a, S: Store> {
    pub(super) po_cell: RefCell<PostOrderIpldIter>,
    pub(super) stack: Vec<Ipld>,
    pub(super) store: &'a S,
}

impl<'a, S: Store> Quiet<'a, S> {
    pub fn new(ipld: Ipld, store: &'a S) -> Self {
        Quiet {
            po_cell: RefCell::new(PostOrderIpldIter::from(ipld)),
            stack: vec![],
            store,
        }
    }
}

impl<'a, S: Store> Iterator for Quiet<'a, S> {
    type Item = Result<Ipld, Cid>;

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
                        return Some(Err(cid));
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

        self.stack.pop().map(Ok)
    }
}
