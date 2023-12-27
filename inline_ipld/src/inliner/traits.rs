//! Traits for inlining [`Ipld`]
use crate::ipld::inlined::InlineIpld;
use crate::store::traits::Store;
use libipld::{Cid, Ipld};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A trait for inlining [`Ipld`]
pub trait Inliner<'a>: Iterator<Item = Ipld> {
    /// Unblock a stuck [`Iterator`].
    ///
    /// This is generally achieved by pushing some [`Ipld`] into the [`Inliner`]'s processing queue.
    ///
    /// Roughly "skipping the line" with some new `Ipld` to directly edit the output during DAG construction.
    /// This is especially helpful for resolving missing [`Ipld`] segments without permanently storing it.
    ///
    /// # Arguments
    ///
    /// * `self` - The [`Inliner`]
    /// * `ipld` - The [`Ipld`] to push into procesisng queue
    fn resolve(&mut self, ipld: Ipld);

    /// Run the [`Inliner`] until completion or unable to progress
    ///
    /// # Examples
    ///
    /// ```
    /// # use inline_ipld::{
    /// #   inliner::{at_least_once::AtLeastOnce, traits::Inliner},
    /// #   store::{
    /// #     traits::Store,
    /// #     memory::MemoryStore
    /// #   }
    /// # };
    /// # use libipld::{ipld, Cid, cid::{CidGeneric, Version}};
    /// # use libipld_cbor::DagCborCodec;
    /// # use multihash::Code::Sha2_256;
    /// # use pretty_assertions::assert_eq;
    /// #
    /// let store = MemoryStore::new(); // NOTE: completely blank through entire example
    /// let missing_cid: Cid = CidGeneric::try_from(
    ///     "bafyreickxqyrg7hhhdm2z24kduovd4k4vvbmfmenzn7nc6pxg6qzjm2v44".to_string(),
    /// )
    /// .unwrap();
    ///
    /// let mut inliner = AtLeastOnce::new(ipld!({"a": 1, "b": missing_cid}));
    /// assert!(inliner.run(&store).unwrap().is_err());
    /// ```
    fn run<S: Store + ?Sized>(
        &'a mut self,
        store: &S,
    ) -> Option<Result<InlineIpld, Stuck<'a, Self>>>;

    /// Manually convert an [`Inliner`] to a [`Stuck`]
    ///
    /// # Arguments
    ///
    /// * `self` - The [`Inliner`]
    /// * `needs` - The [`Cid`] that's required in order to continue
    ///
    /// # Examples
    ///
    /// ```
    /// # use inline_ipld::{
    /// #   inliner::{
    /// #     at_least_once::AtLeastOnce,
    /// #     traits::{Inliner, Stuck}
    /// #   },
    /// #   store::{
    /// #     traits::Store,
    /// #     memory::MemoryStore
    /// #   }
    /// # };
    /// # use libipld::{ipld, Ipld, cid::{CidGeneric, Version}};
    /// # use libipld_cbor::DagCborCodec;
    /// # use multihash::Code::Sha2_256;
    /// # use std::str::FromStr;
    /// # use pretty_assertions::assert_eq;
    /// #
    /// let mut store = MemoryStore::new();
    /// let cid = FromStr::from_str("bafyreickxqyrg7hhhdm2z24kduovd4k4vvbmfmenzn7nc6pxg6qzjm2v44").unwrap();
    ///
    /// let mut inliner = AtLeastOnce::new(Ipld::Null);
    /// assert_eq!(inliner.stuck_at(cid).needs, cid);
    /// ```
    fn stuck_at(&'a mut self, needs: Cid) -> Stuck<'a, Self> {
        Stuck {
            needs,
            inliner: self,
        }
    }
}

/// Error state if a [`Cid`] is not available from the [`Inliner`]'s [`Store`]
///
/// This struct can be [resolved][Stuck::resolve] to continue inlining.
#[derive(PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Desasassaerialize, Serialize))]
pub struct Stuck<'a, I: Inliner<'a> + ?Sized> {
    inliner: &'a mut I,

    /// The [`Cid`] required for the [`Inliner`] to continue
    pub needs: Cid,
}

impl<'a, I: Inliner<'a> + ?Sized> Stuck<'a, I> {
    /// Fill the missig [`Ipld`] in-place, and add it to the [`Store`]
    ///
    /// # Examples
    ///
    /// ```
    /// # use inline_ipld::{
    /// #   inliner::{at_least_once::AtLeastOnce, traits::Inliner},
    /// #   store::{
    /// #     traits::Store,
    /// #     memory::MemoryStore
    /// #   }
    /// # };
    /// # use libipld::{ipld, Ipld, cid::{CidGeneric, Version}, Cid};
    /// # use libipld_cbor::DagCborCodec;
    /// # use multihash::Code::Sha2_256;
    /// # use std::str::FromStr;
    /// # use pretty_assertions::assert_eq;
    /// #
    /// let mut store = MemoryStore::new();
    /// let cid: Cid = FromStr::from_str("bafyreihscx57i276zr5pgnioa5omevods6eseu5h4mllmow6csasju6eqi").unwrap();
    /// let expected = ipld!({"a": 1, "b": {"/": {"link": cid, "data": [1, 2, 3]}}});
    ///
    /// let mut observed = None;
    /// if let Some(Err(mut stuck)) = AtLeastOnce::new(ipld!({"a": 1, "b": cid})).run(&store) {
    ///   observed = Some(stuck.resolve(ipld!([1, 2, 3]), &mut store).run(&store).expect("A").expect("B"));
    /// }
    ///
    /// /// The IPLD is inlined
    /// assert_eq!(observed.unwrap(), expected);
    ///
    /// // The IPLD is now stored
    /// assert_eq!(store.get(&cid).unwrap(), &ipld!([1, 2, 3]));
    /// ```
    pub fn resolve<S: Store + ?Sized>(&'a mut self, ipld: Ipld, store: &mut S) -> &'a mut I {
        store.put_keyed(self.needs, ipld.clone());
        self.stub(ipld)
    }

    /// Fill the missig [`Ipld`] in-place, but do not add it to the [`Store`]
    ///
    /// # Examples
    ///
    /// ```
    /// # use inline_ipld::{
    /// #   inliner::{at_least_once::AtLeastOnce, traits::Inliner},
    /// #   store::{
    /// #     traits::Store,
    /// #     memory::MemoryStore
    /// #   }
    /// # };
    /// # use libipld::{ipld, Ipld, cid::{CidGeneric, Version}, Cid};
    /// # use libipld_cbor::DagCborCodec;
    /// # use multihash::Code::Sha2_256;
    /// # use pretty_assertions::assert_eq;
    /// # use std::str::FromStr;
    /// #
    /// let mut store = MemoryStore::new();
    /// let cid: Cid = FromStr::from_str("bafyreihscx57i276zr5pgnioa5omevods6eseu5h4mllmow6csasju6eqi").unwrap();
    /// let expected = ipld!({"a": 1, "b": {"/": {"link": cid, "data": [1, 2, 3]}}});
    ///
    /// let mut observed = None;
    /// if let Some(Err(mut stuck)) = AtLeastOnce::new(ipld!({"a": 1, "b": cid})).run(&store) {
    ///   observed = Some(stuck.stub(ipld!([1, 2, 3])).run(&store).expect("A").expect("B"));
    /// }
    ///
    /// assert_eq!(observed.unwrap(), expected);
    /// ```
    pub fn stub(&'a mut self, ipld: Ipld) -> &'a mut I {
        self.inliner
            .resolve(InlineIpld::wrap(self.needs.clone(), ipld).into());

        self.inliner
    }

    /// Ignore the stuck [`Cid`] to return to normal [`Inliner`] operation
    ///
    /// This function skips inlining, and leaves the [`Cid`] as a Link.
    ///
    /// # Examples
    ///
    /// ```
    /// # use inline_ipld::{
    /// #   inliner::{at_least_once::AtLeastOnce, traits::Inliner},
    /// #   store::{
    /// #     traits::Store,
    /// #     memory::MemoryStore
    /// #   }
    /// # };
    /// # use libipld::{ipld, Ipld, cid::{CidGeneric, Version}, Cid};
    /// # use libipld_cbor::DagCborCodec;
    /// # use multihash::Code::Sha2_256;
    /// # use std::str::FromStr;
    /// # use pretty_assertions::assert_eq;
    /// #
    /// let mut store = MemoryStore::new();
    /// let cid: Cid = FromStr::from_str("bafyreihscx57i276zr5pgnioa5omevods6eseu5h4mllmow6csasju6eqi").unwrap();
    /// let expected = ipld!({"a": 1, "b": cid});
    ///
    /// if let Some(Err(mut stuck)) = AtLeastOnce::new(ipld!({"a": 1, "b": cid})).run(&mut store) {
    ///   assert_eq!(stuck.needs, cid);
    ///   let result = stuck.ignore().run(&store);
    ///   assert_eq!(result.unwrap().unwrap(), expected);
    /// }
    /// ```
    pub fn ignore(&'a mut self) -> &'a mut I {
        self.inliner.resolve(Ipld::Link(self.needs.clone()));
        self.inliner
    }
}
