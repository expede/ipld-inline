//! An inliner that stops if it is unable to find a subgraph
use super::{naive::Naive, traits::Inliner};
use crate::store::traits::Store;
use libipld::{cid::Cid, ipld::Ipld};

#[cfg(feature = "serde")]
use serde::Serialize;

/// [`Ipld`] inliner that only inlines a [`Cid`] once
///
/// Unlike [`AtMostOnce`][crate::inliner::at_most_once::AtMostOnce], this inliner will stops if the [`Cid`] is not availale from the [`Store`].
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct AtLeastOnce<'a, S: Store + ?Sized> {
    naive: Naive<'a, S>,
    needs: Option<Cid>,
}

impl<'a, S: Store + ?Sized> AtLeastOnce<'a, S> {
    /// Initialize a new [`AtLeastOnce`]
    pub fn new(ipld: Ipld, store: &'a mut S) -> Self {
        AtLeastOnce {
            naive: Naive::new(ipld, store),
            needs: None,
        }
    }
}

impl<'a, S: Store + ?Sized> From<Naive<'a, S>> for AtLeastOnce<'a, S> {
    fn from(naive: Naive<'a, S>) -> Self {
        AtLeastOnce { naive, needs: None }
    }
}

impl<'a, S: Store + ?Sized> From<AtLeastOnce<'a, S>> for Naive<'a, S> {
    fn from(exactly_once: AtLeastOnce<'a, S>) -> Self {
        exactly_once.naive
    }
}

impl<'a, S: Store + ?Sized> Iterator for AtLeastOnce<'a, S> {
    type Item = Result<Ipld, Cid>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.needs.is_some() {
            return None;
        }

        match self.naive.next() {
            None => None,
            Some(Err(cid)) => {
                self.needs = Some(cid);
                Some(Err(cid))
            }
            Some(Ok(ipld)) => Some(Ok(ipld.clone())),
        }
    }
}

impl<'a, S: Store + ?Sized> Inliner<'a> for AtLeastOnce<'a, S> {
    fn store(&mut self, cid: &Cid, ipld: &Ipld) {
        self.naive.store.put_keyed(cid.clone(), ipld.clone())
    }

    fn interject(&mut self, ipld: &Ipld) {
        self.needs = None;
        self.naive.push(ipld.clone())
    }
}

// FIXME writes tests
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::store::memory::MemoryStore;
//     use libipld::ipld;
//     use pretty_assertions::assert_eq;
//
//     #[test]
//     fn happy_little_test() {
//         let mut store = MemoryStore::new();
//         let mut c = AtLeastOnce::new(ipld!([1, 2, 3]), &mut store);
//         match c.tryme() {
//             Ok(_) => assert!(true),
//             Err(_) => assert!(true),
//         }
//
//         assert!(true);
//     }
// }
