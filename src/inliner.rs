use crate::iterator::post_order::PostOrderIpldIter;
use crate::store::Store;
use libipld::{cid::Cid, ipld::Ipld};
use std::collections::BTreeMap;
use std::ops::ControlFlow;
use std::ops::ControlFlow::{Break, Continue};

#[derive(Clone, Debug)]
pub struct Inliner<'a, S: Store> {
    iterator: PostOrderIpldIter<'a>,
    store: S,
}

// FIXME iterator?

impl<'a, S: Store> Inliner<'a, S> {
    pub fn new(ipld: &'a Ipld, store: S) -> Self {
        Inliner {
            iterator: PostOrderIpldIter::from(ipld),
            store,
        }
    }

    pub fn try_inline(&'a mut self) -> State<'a, S> {
        let folded: ControlFlow<&Cid, Vec<Ipld>> =
            self.iterator.try_fold(vec![], |mut acc, node| match node {
                Ipld::Map(btree) => {
                    let new_btree = btree.keys().cloned().zip(acc).collect();
                    Continue(vec![Ipld::Map(new_btree)])
                }
                Ipld::List(vec) => {
                    let new_vec = acc.iter().take(vec.len()).cloned().collect();
                    acc.push(Ipld::List(new_vec));
                    Continue(acc)
                }
                link @ Ipld::Link(cid) => {
                    if let Some(ipld) = self.store.get(cid) {
                        let mut inner = BTreeMap::new();
                        inner.insert("link".to_string(), link.clone());
                        inner.insert("data".to_string(), ipld.clone());

                        let mut outer = BTreeMap::new();
                        outer.insert("/".to_string(), Ipld::Map(inner));

                        acc.push(Ipld::Map(outer));
                        Continue(acc)
                    } else {
                        Break(cid)
                    }
                }
                node => {
                    acc.push(node.clone());
                    Continue(acc)
                }
            });

        match folded {
            Break(missing_cid) => State::Stuck(StuckAt {
                need: missing_cid,
                inliner: self,
            }),
            Continue(mut x) => {
                State::Done(x.pop().expect("should have exactly one item on the stack"))
            }
        }
    }
}

pub enum State<'a, S: Store> {
    Done(Ipld),
    Stuck(StuckAt<'a, S>),
}

pub struct StuckAt<'a, S: Store> {
    need: &'a Cid,
    inliner: &'a mut Inliner<'a, S>,
}

impl<'a, S: Store> StuckAt<'a, S> {
    pub fn wants(&'a self) -> &'a Cid {
        self.need
    }

    pub fn ignore(&'a self) -> &'a Inliner<'a, S> {
        self.inliner
    }

    pub fn continue_with(&'a mut self, ipld: &'a Ipld) -> &mut Inliner<'_, S> {
        let mut new_inliner = self.inliner.clone(); // FIXME
        new_inliner.iterator.impose_next(ipld);
        self.inliner
    }
}
