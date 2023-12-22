//! An inliner that stops if it is unable to find a subgraph

use super::quiet::Quiet;
use crate::store::traits::Store;
use libipld::{cid::Cid, ipld, ipld::Ipld};

/////////////
// Structs //
/////////////

/// [`Ipld`] inliner that only inlines a [`Cid`] once
///
/// Unlike [`AtMostOnce`][crate::inliner::at_most_once::AtMostOnce], this inliner will stops if the [`Cid`] is not availale from the [`Store`].
#[derive(PartialEq, Debug)]
pub struct ExactlyOnce<'a, S: Store + ?Sized> {
    quiet: Quiet<'a, S>,
    stuck_at: Option<Cid>,
}

/// Error state if a [`Cid`] is not available from the [`ExactlyOnce`]'s [`Store`]
///
/// This struct can be [resolved][Stuck::resolve] to continue inlining.
#[derive(PartialEq, Debug)]
pub struct Stuck<'a, S: Store + ?Sized> {
    pub needs: Cid,
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

// FIXME with_stuck or similar?

impl<'a, S: Store + ?Sized> Stuck<'a, S> {
    /// Fill the missig [`Ipld`] in-place, and add it to the [`Store`]
    ///
    /// # Examples
    ///
    /// ```
    /// # use ipld_inline::{
    /// #   inliner::exactly_once::ExactlyOnce,
    /// #   store::{
    /// #     traits::Store,
    /// #     memory::MemoryStore
    /// #   }
    /// # };
    /// # use libipld::{ipld, Ipld, cid::{CidGeneric, Version}, Cid};
    /// # use libipld_cbor::DagCborCodec;
    /// # use multihash::Code::Sha2_256;
    /// # use std::str::FromStr;
    /// #
    /// let mut store = MemoryStore::new();
    /// let cid: Cid = FromStr::from_str("bafyreihscx57i276zr5pgnioa5omevods6eseu5h4mllmow6csasju6eqi").unwrap();
    /// let expected = ipld!({"a": 1, "b": {"/": {"link": cid, "data": [1, 2, 3]}}});
    ///
    /// let mut observed = None;
    /// if let Some(Err(mut stuck)) = ExactlyOnce::new(ipld!({"a": 1, "b": cid}), &mut store).run() {
    ///   observed = Some(stuck.resolve(ipld!([1, 2, 3])).run().expect("A").expect("B"));
    /// }
    /// assert_eq!(observed, Some(expected));
    /// assert_eq!(store.get(&cid).unwrap(), &ipld!([1, 2, 3]));
    /// ```
    pub fn resolve(&'a mut self, ipld: Ipld) -> &'a mut ExactlyOnce<'a, S> {
        self.iterator
            .quiet
            .store
            .put_keyed(self.needs, ipld.clone());

        self.stub(ipld)
    }

    /// Fill the missig [`Ipld`] in-place, but do not add it to the [`Store`]
    ///
    /// # Examples
    ///
    /// ```
    /// # use ipld_inline::{
    /// #   inliner::exactly_once::ExactlyOnce,
    /// #   store::{
    /// #     traits::Store,
    /// #     memory::MemoryStore
    /// #   }
    /// # };
    /// # use libipld::{ipld, Ipld, cid::{CidGeneric, Version}, Cid};
    /// # use libipld_cbor::DagCborCodec;
    /// # use multihash::Code::Sha2_256;
    /// # use std::str::FromStr;
    /// #
    /// let mut store = MemoryStore::new();
    /// let cid: Cid = FromStr::from_str("bafyreihscx57i276zr5pgnioa5omevods6eseu5h4mllmow6csasju6eqi").unwrap();
    /// let expected = ipld!({"a": 1, "b": {"/": {"link": cid, "data": [1, 2, 3]}}});
    ///
    /// let mut observed = None;
    /// if let Some(Err(mut stuck)) = ExactlyOnce::new(ipld!({"a": 1, "b": cid}), &mut store).run() {
    ///   observed = Some(stuck.stub(ipld!([1, 2, 3])).run().expect("A").expect("B"));
    /// }
    /// assert_eq!(observed, Some(expected));
    /// ```
    pub fn stub(&'a mut self, ipld: Ipld) -> &'a mut ExactlyOnce<'a, S> {
        self.iterator.quiet.push(
            ipld!({ // FIXME break out a "inline chunk" helper. Maybe just `inline!`?
                "/": {
                    "data": ipld.clone(),
                    "link": self.needs.clone()
                }
            }),
        );

        self.iterator.stuck_at = None;
        self.iterator
    }

    /// Ignore the stuck [`Cid`] to return to normal [`ExactlyOnce`] operation
    ///
    /// This function skips inlining, and leaves the [`Cid`] as a Link.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ipld_inline::{
    /// #   inliner::exactly_once::ExactlyOnce,
    /// #   store::{
    /// #     traits::Store,
    /// #     memory::MemoryStore
    /// #   }
    /// # };
    /// # use libipld::{ipld, Ipld, cid::{CidGeneric, Version}, Cid};
    /// # use libipld_cbor::DagCborCodec;
    /// # use multihash::Code::Sha2_256;
    /// # use std::str::FromStr;
    /// #
    /// let mut store = MemoryStore::new();
    /// let cid: Cid = FromStr::from_str("bafyreihscx57i276zr5pgnioa5omevods6eseu5h4mllmow6csasju6eqi").unwrap();
    /// let expected = ipld!({"a": 1, "b": cid});
    ///
    /// let mut observed = None;
    /// if let Some(Err(mut stuck)) = ExactlyOnce::new(ipld!({"a": 1, "b": cid}), &mut store).run() {
    ///   observed = Some(stuck.ignore().run().unwrap().unwrap());
    /// }
    /// assert_eq!(observed, Some(expected));
    /// ```
    pub fn ignore(&'a mut self) -> &'a mut ExactlyOnce<'a, S> {
        self.iterator.quiet.push(Ipld::Link(self.needs.clone()));
        self.iterator.stuck_at = None;
        self.iterator
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

    /// The prinmary interface for [`ExactlyOnce`]. This runs the inliner.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ipld_inline::{
    /// #   inliner::exactly_once::ExactlyOnce,
    /// #   store::{
    /// #     traits::Store,
    /// #     memory::MemoryStore
    /// #   }
    /// # };
    /// # use libipld::{ipld, cid::{CidGeneric, Version}};
    /// # use libipld_cbor::DagCborCodec;
    /// # use multihash::Code::Sha2_256;
    /// #
    /// let mut store = MemoryStore::new();
    /// let cid = store.put(ipld!([1, 2, 3]), DagCborCodec, &Sha2_256, Version::V1).unwrap();
    ///
    /// let mut exactly_once = ExactlyOnce::new(ipld!({"a": 1, "b": cid}), &mut store);
    /// let expected = ipld!({"a": 1, "b": {"/": {"link": cid, "data": [1, 2, 3]}}});
    ///
    /// assert_eq!(exactly_once.run().unwrap().unwrap(), expected);
    /// ```
    /// FIXME the above can't compare in the eq
    /// FIXME show the err case
    pub fn run(&'a mut self) -> Option<Result<Ipld, Stuck<'a, S>>> {
        match self.last() {
            Some(Ok(ipld)) => Some(Ok(ipld)),
            Some(Err(cid)) => Some(Err(Stuck {
                needs: cid,
                iterator: self,
            })),
            None => None,
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
