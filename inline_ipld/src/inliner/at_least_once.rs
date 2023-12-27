//! Inline each subgraph at _least_ once (without deduplication)
use super::traits::{Inliner, Stuck};
use crate::{
    ipld::inlined::InlineIpld, iterator::post_order::PostOrderIpldIter, store::traits::Store,
};
use libipld::{cid::Cid, ipld::Ipld};
use std::collections::BTreeMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Inline directly, stopping at missing nodes, but without deduplication.
///
/// This inlining strategy tries its best, but:
/// - Doesn't attempt to deduplicate DAGs
/// - Doesn't stop if a [`Cid`] is not available in the attached [`Store`]
///
/// In general, you should prefer the use of the [`Inliner`] interface, over [`Iterator`].
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct AtLeastOnce {
    po: PostOrderIpldIter,
    stack: Vec<Ipld>,
    needs: Option<Cid>,
}

impl AtLeastOnce {
    // FIXME pass by ref?
    /// Initialize a new [`AtLeastOnce`] inliner
    pub fn new(ipld: Ipld) -> Self {
        ipld.into()
    }
}

impl From<Ipld> for AtLeastOnce {
    // FIXME pass by ref?
    fn from(ipld: Ipld) -> Self {
        AtLeastOnce {
            po: PostOrderIpldIter::from(ipld),
            stack: vec![],
            needs: None,
        }
    }
}

impl Iterator for AtLeastOnce {
    type Item = Ipld;

    fn next(&mut self) -> Option<Self::Item> {
        self.needs?;
        self.po.next()
    }
}

impl<'a> Inliner<'a> for AtLeastOnce {
    // FIXME just rename as something closer to `resolve` or `unstick`?
    fn resolve(&mut self, ipld: Ipld) {
        self.stack.push(ipld);
        self.needs = None;
    }

    fn run<S: Store + ?Sized>(
        &'a mut self,
        store: &S,
    ) -> Option<Result<InlineIpld, Stuck<'a, Self>>> {
        for node in &mut self.po {
            match node {
                Ipld::Link(cid) => {
                    if let Ok(ipld) = store.get(&cid) {
                        let mut inner = BTreeMap::new();
                        inner.insert("link".to_string(), Ipld::Link(cid));
                        inner.insert("data".to_string(), ipld.clone());

                        let mut outer = BTreeMap::new();
                        outer.insert("/".to_string(), Ipld::Map(inner));

                        self.stack.push(Ipld::Map(outer));
                    } else {
                        return Some(Err(self.stuck_at(cid)));
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

        // Top of the inlined DAG. `pop` should only be empty if the Iterator was empty
        self.stack.pop().map(|ipld| Ok(InlineIpld::attest(ipld)))
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::store::memory::MemoryStore;
//     use libipld::ipld;
//     use pretty_assertions::assert_eq;
//
//     #[test]
//     fn interject_test() {
//         let mut store = MemoryStore::new();
//         let mut inliner = AtLeastOnce::new(ipld!([1, 2, 3]));
//
//         let cid: Cid =
//             FromStr::from_str("bafyreihscx57i276zr5pgnioa5omevods6eseu5h4mllmow6csasju6eqi")
//                 .unwrap();
//
//         let mut observed = None;
//         if let Some(Err(mut stuck)) = AtLeastOnce::new(ipld!({"a": 1, "b": cid})).run(&mut store) {
//             observed = Some(stuck.ignore().run(&mut store).unwrap().unwrap());
//         }
//
//         assert_eq!(observed.unwrap(), ipld!({"a": 1, "b": cid}););
//     }
// }
