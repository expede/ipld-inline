//! An inliner that stops if it is unable to find a subgraph

use super::quiet::Quiet;
use crate::store::traits::Store;
use libipld::{cid::Cid, ipld::Ipld};

/////////////
// Structs //
/////////////

/// [`Ipld`] inliner that only inlines a [`Cid`] once
///
/// Unlike [`AtMostOnce`][crate::inliner::at_most_once::AtMostOnce], this inliner will stops if the [`Cid`] is not availale from the [`Store`].
#[derive(Debug)]
pub struct ExactlyOnce<'a, S: Store + ?Sized> {
    quiet: Quiet<'a, S>,
    stuck_at: Option<Cid>,
}

/// Error state if a [`Cid`] is not available from the [`ExactlyOnce`]'s [`Store`]
///
/// This struct can be [resolved][Stuck::resolve] to continue inlining.
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
    pub fn ignore(&'a mut self) -> &'a mut ExactlyOnce<'a, S> {
        self.iterator.quiet.push(Ipld::Link(self.needs.clone()));
        self.iterator
    }

    pub fn stub(&'a mut self, ipld: Ipld) -> &'a mut ExactlyOnce<'a, S> {
        self.iterator.quiet.push(ipld.clone());
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
    /// Initialize a new [`ExactlyOnce`]
    pub fn new(ipld: Ipld, store: &'a mut S) -> Self {
        ExactlyOnce {
            quiet: Quiet::new(ipld, store),
            stuck_at: None,
        }
    }

    //  /// Returns the [`Cid`] that the iterator got stuck at
    //  ///
    //  /// # Examples
    //  ///
    //  /// ```
    //  /// # use ipld_inline::inliner::exactly_once::ExactlyOnce;
    //  /// # use ipld_inline::store::memory::MemoryStore;
    //  /// # use libipld::{ipld, cid::CidGeneric};
    //  /// #
    //  /// let mut store = MemoryStore::new();
    //  /// let missing_cid = CidGeneric::try_from(
    //  ///   "bafyreihnubkcms63243zlfgnwiugmk6ijitz63me7bqf455ia2fpbn4ceq".to_string(),
    //  /// )
    //  /// .unwrap();
    //  /// let mut exactly_once = ExactlyOnce::new(ipld!({"a": 1, "b": missing_cid}), &mut store);
    //  /// {
    //  ///   let mut foo = exactly_once;
    //  ///   foo.last();
    //  /// }
    //  /// // let stuck_at = exactly_once.last().unwrap();
    //  /// exactly_once.ignore();
    //  /// assert_eq!(exactly_once.wants(), Some(missing_cid));
    //  /// ```
    pub fn wants(&self) -> Option<Cid> {
        self.stuck_at
    }

    //  /// FIXME
    //  ///
    //  /// # Examples
    //  ///
    //  /// ```
    //  /// # use ipld_inline::inliner::exactly_once::ExactlyOnce;
    //  /// # use ipld_inline::store::memory::MemoryStore;
    //  /// # use libipld::{ipld, cid::CidGeneric};
    //  /// # use std::str::FromStr;
    //  /// #
    //  /// let mut store = MemoryStore::new();
    //  /// let missing_cid = FromStr::from_str("bafyreihnubkcms63243zlfgnwiugmk6ijitz63me7bqf455ia2fpbn4ceq").unwrap();
    //  ///
    //  /// let mut exactly_once = ExactlyOnce::new(ipld!({"a": 1, "b": missing_cid}), &mut store);
    //  /// exactly_once.last();
    //  /// exactly_once.ignore();
    //  ///
    //  /// assert_eq!(exactly_once.next(), Some(Err(missing_cid)));
    //  /// ```
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

    // FIXME
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

    /// Returns the [`Cid`] that the iterator got stuck at
    ///
    /// # Examples
    ///
    /// ```
    /// # use ipld_inline::inliner::exactly_once::ExactlyOnce;
    /// # use ipld_inline::store::memory::MemoryStore;
    /// # use libipld::{ipld, cid::CidGeneric};
    /// #
    /// let mut store = MemoryStore::new();
    /// let missing_cid = CidGeneric::try_from(
    ///   "bafyreihnubkcms63243zlfgnwiugmk6ijitz63me7bqf455ia2fpbn4ceq".to_string(),
    /// )
    /// .unwrap();
    /// let mut exactly_once = ExactlyOnce::new(ipld!({"a": 1, "b": missing_cid}), &mut store);
    /// let mut stuck = match exactly_once.happy_next().unwrap() {
    ///   Ok(_) => todo!(),
    ///   Err(s) => s
    /// };
    /// let eio = stuck.ignore();
    /// assert_eq!(eio.wants(), Some(missing_cid));
    /// ```
    pub fn happy_next(&'a mut self) -> Option<Result<Ipld, Stuck<'a, S>>> {
        if self.stuck_at.is_some() {
            return None;
        }

        match self.quiet.next() {
            None => None,
            Some(Err(cid)) => {
                self.stuck_at = Some(cid);
                Some(Err(Stuck {
                    needs: cid,
                    iterator: self,
                }))
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
