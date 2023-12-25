// FIXME this does NOT have exactly once semantics

//! An inliner that stops if it is unable to find a subgraph
use super::{naive::Naive, traits::Inliner};
use crate::store::traits::Store;
use libipld::{cid::Cid, ipld::Ipld};

/// [`Ipld`] inliner that only inlines a [`Cid`] once
///
/// Unlike [`AtMostOnce`][crate::inliner::at_most_once::AtMostOnce], this inliner will stops if the [`Cid`] is not availale from the [`Store`].
#[derive(Debug, PartialEq)]
pub struct ExactlyOnce<'a, S: Store + ?Sized> {
    naive: Naive<'a, S>,
    needs: Option<Cid>,
}

impl<'a, S: Store + ?Sized> ExactlyOnce<'a, S> {
    /// Initialize a new [`ExactlyOnce`]
    pub fn new(ipld: Ipld, store: &'a mut S) -> Self {
        ExactlyOnce {
            naive: Naive::new(ipld, store),
            needs: None,
        }
    }
}

impl<'a, S: Store + ?Sized> From<Naive<'a, S>> for ExactlyOnce<'a, S> {
    fn from(naive: Naive<'a, S>) -> Self {
        ExactlyOnce { naive, needs: None }
    }
}

impl<'a, S: Store + ?Sized> From<ExactlyOnce<'a, S>> for Naive<'a, S> {
    fn from(exactly_once: ExactlyOnce<'a, S>) -> Self {
        exactly_once.naive
    }
}

impl<'a, S: Store + ?Sized> Iterator for ExactlyOnce<'a, S> {
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

impl<'a, S: Store + ?Sized> Inliner<'a> for ExactlyOnce<'a, S> {
    fn store(&mut self, cid: &Cid, ipld: &Ipld) {
        self.naive.store.put_keyed(cid.clone(), ipld.clone()) // FIXME attack of the clone
    }

    fn interject(&mut self, ipld: &Ipld) {
        self.needs = None;
        self.naive.push(ipld.clone()) // FIXME attack of the clones
    }
}

// FIXME
#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::memory::MemoryStore;
    use libipld::ipld;
    use pretty_assertions::assert_eq;

    // FIXME
    #[test]
    fn happy_little_test() {
        let mut store = MemoryStore::new();
        let mut c = ExactlyOnce::new(ipld!([1, 2, 3]), &mut store);
        match c.tryme() {
            Ok(_) => assert!(true),
            Err(_) => assert!(true),
        }

        assert!(true);
    }
}
