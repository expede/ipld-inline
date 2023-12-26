//! Traits for inlining [`Ipld`]
use crate::ipld::inlined::InlineIpld;
use crate::store::traits::Store;
use libipld::{Cid, Ipld};

/// A trait for inlining [`Ipld`]
pub trait Inliner<'a>: Iterator<Item = Result<Ipld, Cid>> {
    // These are shared with Stuck... should that get extracted?

    // /// Store some [`Ipld`] in the [`Inliner`]'s state
    // ///
    // /// This is useful for deduplication, avoiding [`Stuck`] states, and so on
    // ///
    // /// # Arguments
    // ///
    // /// * `self` - The [`Inliner`]
    // /// * `cid` - The [`Cid`] to use as a key. It's relationship to `ipld` is not checked.
    // /// * `ipld` - The [`Ipld`] to store
    // ///
    // /// # Examples
    // ///
    // /// FIXME write doctest
    // /// # use pretty_assertions::assert_eq;
    // fn store(&mut self, cid: &Cid, ipld: &Ipld);

    /// Push some [`Ipld`] into the processing queue state of the [`Inliner`]
    ///
    /// Roughly "skipping the line" with some new `Ipld` to directly edit the output during DAG construction.
    /// This is especially helpful for resolving missing [`Ipld`] segments without permanently storing it.
    ///
    /// # Arguments
    ///
    /// * `self` - The [`Inliner`]
    /// * `ipld` - The [`Ipld`] to push into procesisng queue
    ///
    /// # Examples
    ///
    /// FIXME write doctest
    /// # use pretty_assertions::assert_eq;
    fn interject(&mut self, ipld: &Ipld); // NOTE TO SELF: in definition, set `needs = None`

    /// Run the [`Inliner`] until completion or unable to progress
    ///
    /// # Examples
    ///
    /// ```
    /// # use inline_ipld::{
    /// #   inliner::exactly_once::ExactlyOnce,
    /// #   store::{
    /// #     traits::Store,
    /// #     memory::MemoryStore
    /// #   }
    /// # };
    /// # use libipld::{ipld, cid::{CidGeneric, Version}};
    /// # use libipld_cbor::DagCborCodec;
    /// # use multihash::Code::Sha2_256;
    /// # use pretty_assertions::assert_eq;
    /// #
    /// let mut store = MemoryStore::new();
    /// let cid = store.put(ipld!([1, 2, 3]), DagCborCodec, &Sha2_256, Version::V1).unwrap();
    ///
    /// let mut exactly_once = ExactlyOnce::new(ipld!({"a": 1, "b": cid}), &mut store);
    /// let expected = ipld!({"a": 1, "b": {"/": {"link": cid, "data": [1, 2, 3]}}});
    ///
    /// assert_eq!(exactly_once.run().unwrap().unwrap(), expected);
    /// ```
    ///
    /// FIXME show the err case
    fn run(&'a mut self, store: &dyn Store) -> Option<Result<InlineIpld, Stuck<'a, Self>>> {
        match self.last()? {
            Ok(ipld) => Some(Ok(InlineIpld::already_inlined(ipld))),
            Err(needs) => Some(Err(self.stuck_at(needs))),
        }
    }

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
    /// #     exactly_once::ExactlyOnce,
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
    /// let mut inliner = ExactlyOnce::new(Ipld::Null, &mut store);
    /// assert_eq!(inliner.stuck_at(cid).needs, cid);
    /// ```
    fn stuck_at(&'a mut self, needs: Cid) -> Stuck<'a, Self> {
        Stuck {
            needs,
            inliner: self,
        }
    }
}

/// Error state if a [`Cid`] is not available from the [`ExactlyOnce`]'s [`Store`]
///
/// This struct can be [resolved][Stuck::resolve] to continue inlining.
#[derive(PartialEq, Debug)]
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
    ///
    /// /// The IPLD is inlined
    /// assert_eq!(observed, Some(expected));
    ///
    /// // The IPLD is now stored
    /// assert_eq!(store.get(&cid).unwrap(), &ipld!([1, 2, 3]));
    /// ```
    pub fn resolve(&'a mut self, ipld: &Ipld) -> &'a mut I {
        self.inliner.store(&self.needs, ipld);
        self.stub(ipld)
    }

    /// Fill the missig [`Ipld`] in-place, but do not add it to the [`Store`]
    ///
    /// # Examples
    ///
    /// ```
    /// # use inline_ipld::{
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
    pub fn stub(&'a mut self, ipld: &Ipld) -> &'a mut I {
        self.inliner
            .interject(&InlineIpld::wrap(self.needs.clone(), ipld.clone()).into());

        self.inliner
    }

    /// Ignore the stuck [`Cid`] to return to normal [`ExactlyOnce`] operation
    ///
    /// This function skips inlining, and leaves the [`Cid`] as a Link.
    ///
    /// # Examples
    ///
    /// ```
    /// # use inline_ipld::{
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
    pub fn ignore(&'a mut self) -> &'a mut I {
        self.inliner.interject(&Ipld::Link(self.needs.clone()));
        self.inliner
    }
}
