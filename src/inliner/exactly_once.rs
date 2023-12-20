//! An inliner that stops if it is unable to find a subgraph

use super::quiet::Quiet;
use crate::store::traits::Store;
use libipld::{cid::Cid, ipld::Ipld};

/////////////
// Structs //
/////////////

#[derive(Debug)]
pub struct ExactlyOnce<'a, S: Store + ?Sized> {
    quiet: Quiet<'a, S>,
    stuck_at: Option<Cid>,
}

#[derive(Debug)]
pub struct Stuck<'a, S: Store + ?Sized> {
    needs: Cid,
    iterator: &'a mut ExactlyOnce<'a, S>,
}

//////////////////////////////
// Standard Implementations //
//////////////////////////////

impl<'a, S: Store + ?Sized> TryFrom<&'a mut ExactlyOnce<'a, S>> for Stuck<'a, S> {
    type Error = (); // FIXME

    fn try_from(eo: &'a mut ExactlyOnce<'a, S>) -> Result<Stuck<'a, S>, Self::Error> {
        match eo.stuck_at {
            Some(cid) => Ok(Stuck {
                needs: cid,
                iterator: eo,
            }),
            None => Err(()),
        }
    }
}

impl<'a, S: Store + ?Sized> From<Quiet<'a, S>> for ExactlyOnce<'a, S> {
    fn from(quiet: Quiet<'a, S>) -> Self {
        ExactlyOnce {
            quiet,
            stuck_at: None,
        }
    }
}

impl<'a, S: Store + ?Sized> From<ExactlyOnce<'a, S>> for Quiet<'a, S> {
    fn from(exactly_once: ExactlyOnce<'a, S>) -> Self {
        exactly_once.quiet
    }
}

impl<'a, S: Store + ?Sized> Iterator for ExactlyOnce<'a, S> {
    type Item = Result<Ipld, Cid>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stuck_at.is_some() {
            return None;
        }

        match self.quiet.next() {
            None => None,
            Some(Err(cid)) => {
                self.stuck_at = Some(cid);
                Some(Err(cid))
            }
            Some(Ok(ipld)) => Some(Ok(ipld.clone())),
        }
    }
}

////////////////////////////
// Custom Implementations //
////////////////////////////

impl<'a, S: Store + ?Sized> Stuck<'a, S> {
    pub fn stub(&'a mut self, ipld: Ipld) -> &'a mut ExactlyOnce<'a, S> {
        self.iterator.quiet.push(ipld.clone()); // FIXME Needs tests
        self.iterator
    }

    pub fn resolve(&'a mut self, ipld: Ipld) -> &'a mut ExactlyOnce<'a, S> {
        self.iterator
            .quiet
            .store
            .put_keyed(self.needs, ipld.clone());

        self.stub(ipld)
    }
}

impl<'a, S: Store + ?Sized> ExactlyOnce<'a, S> {
    pub fn new(ipld: Ipld, store: &'a mut S) -> Self {
        ExactlyOnce {
            quiet: Quiet::new(ipld, store),
            stuck_at: None,
        }
    }

    pub fn wants(&self) -> Option<Cid> {
        self.stuck_at
    }

    pub fn ignore(&mut self) -> Option<()> {
        self.stuck_at.map(|cid| {
            self.quiet.push(Ipld::Link(cid));
            self.stuck_at = None;
        })
    }

    // FIXME don't 100% love this
    pub fn resolve(&mut self, ipld: Ipld) -> Option<()> {
        self.stuck_at.map(|_| {
            self.quiet.push(ipld);
            self.stuck_at = None;
        })
    }

    pub fn tryme(&'a mut self) -> Result<(), Stuck<'a, S>> {
        self.last();
        match self.stuck_at {
            None => Ok(()),
            Some(cid) => Err(Stuck {
                needs: cid,
                iterator: self,
            }),
        }
    }

    //     fn happy_next(&'a mut self) -> Option<Result<Ipld, Stuck<'a, S>>> {
    //         if self.stuck_at.is_some() {
    //             return None;
    //         }
    //
    //         match self.quiet.next() {
    //             None => None,
    //             Some(Err(cid)) => {
    //                 self.stuck_at = Some(cid);
    //                 Some(Err(Stuck {
    //                     needs: cid,
    //                     it: self,
    //                 }))
    //             }
    //             Some(Ok(ipld)) => Some(Ok(ipld.clone())),
    //         }
    //     }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::memory::MemoryStore;
    use libipld::ipld;

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
