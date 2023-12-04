use super::quiet::Quiet;
use crate::store::traits::Store;
use libipld::{cid::Cid, ipld::Ipld};

#[derive(Clone, Debug)]
pub struct ExactlyOnce<'a, S: Store + ?Sized> {
    quiet: Quiet<'a, S>,
    stuck_at: Option<Cid>,
}

pub struct Stuck<'a, S: Store + ?Sized> {
    needs: Cid,
    it: &'a mut ExactlyOnce<'a, S>,
}

impl<'a, S: Store + ?Sized> Stuck<'a, S> {
    pub fn stub(&'a mut self, ipld: Ipld) -> &'a mut ExactlyOnce<'a, S> {
        // let needs = self.needs;
        // FIXME probablyt want to add thsi to the store, too, but changes the store mut...
        // self.it.quiet.store.put_keyed(needs, ipld.clone());
        self.it.quiet.push(ipld.clone()); // Needs tests
        self.it
    }
}

// TODO TryInto Stuck for ExactlyOnce
impl<'a, S: Store + ?Sized> TryFrom<&'a mut ExactlyOnce<'a, S>> for Stuck<'a, S> {
    type Error = ();

    fn try_from(eo: &'a mut ExactlyOnce<'a, S>) -> Result<Stuck<'a, S>, Self::Error> {
        match eo.stuck_at {
            Some(cid) => Ok(Stuck { needs: cid, it: eo }),
            None => Err(()),
        }
    }
}

impl<'a, S: Store + ?Sized> ExactlyOnce<'a, S> {
    pub fn new(ipld: Ipld, store: &'a S) -> Self {
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
                it: self,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::memory::MemoryStore;
    use libipld::ipld;

    #[test]
    fn happy_little_test() {
        let store = MemoryStore::new();
        let mut c = ExactlyOnce::new(ipld!([1, 2, 3]), &store);
        match c.tryme() {
            Ok(_) => assert!(true),
            Err(_) => assert!(true),
        }

        assert!(true);
    }
}
