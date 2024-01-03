//! Traits for inlining [`Ipld`]
use crate::{ipld::InlineIpld, store::Store};
use libipld::{Cid, Ipld};
use std::ops::DerefMut;

/// A trait for inlining [`Ipld`]
pub trait Inliner {
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
    /// #     MemoryStore
    /// #   }
    /// # };
    /// # use libipld::{ipld, Cid, cid::{CidGeneric, Version}};
    /// # use libipld::cbor::DagCborCodec;
    /// # use multihash::Code::Sha2_256;
    /// # use pretty_assertions::assert_eq;
    /// #
    /// let store = MemoryStore::new(); // NOTE: completely blank through entire example
    /// let missing_cid: Cid = CidGeneric::try_from(
    ///     "bafyreickxqyrg7hhhdm2z24kduovd4k4vvbmfmenzn7nc6pxg6qzjm2v44".to_string(),
    /// )
    /// .unwrap();
    ///
    /// let dag = ipld!({"a": 1, "b": missing_cid});
    /// let mut inliner = AtLeastOnce::new(&dag);
    /// assert!(inliner.run(&store).unwrap().is_err());
    /// ```
    fn run<S: Store + ?Sized>(self, store: &S) -> Option<Result<InlineIpld, Stuck<Self>>>;

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
    /// #     MemoryStore
    /// #   }
    /// # };
    /// # use libipld::{ipld, Ipld, cid::{CidGeneric, Version}};
    /// # use libipld::cbor::DagCborCodec;
    /// # use multihash::Code::Sha2_256;
    /// # use std::str::FromStr;
    /// # use pretty_assertions::assert_eq;
    /// #
    /// let mut store = MemoryStore::new();
    /// let cid = FromStr::from_str("bafyreickxqyrg7hhhdm2z24kduovd4k4vvbmfmenzn7nc6pxg6qzjm2v44").unwrap();
    ///
    /// let mut inliner = AtLeastOnce::new(&Ipld::Null);
    /// assert_eq!(inliner.stuck_at(cid).needs(), cid);
    /// ```
    fn stuck_at(self, needs: Cid) -> Stuck<Self>
    where
        Self: Sized,
    {
        Stuck {
            needs,
            inliner: Box::new(self),
        }
    }
}

/// Error state if a [`Cid`] is not available from the [`Inliner`]'s [`Store`]
///
/// This struct can be [resolved][Stuck::resolve] to continue inlining.
#[derive(PartialEq, Debug)]
#[cfg_attr(feature = "serde-codec", derive(serde::Deserialize, serde::Serialize))]
pub struct Stuck<I: Inliner + ?Sized> {
    inliner: Box<I>,
    needs: Cid,
}

impl<I: Inliner> Stuck<I> {
    /// Get the [`Cid`] required for the [`Inliner`] to continue
    pub fn needs(&self) -> Cid {
        self.needs
    }

    /// Fill the missig [`Ipld`] in-place, and add it to the [`Store`]
    ///
    /// # Examples
    ///
    /// ```
    /// # use inline_ipld::{
    /// #   inliner::{at_least_once::AtLeastOnce, traits::Inliner},
    /// #   store::{
    /// #     traits::Store,
    /// #     MemoryStore
    /// #   }
    /// # };
    /// # use libipld::{ipld, Ipld, cid::{CidGeneric, Version}, Cid};
    /// # use libipld::cbor::DagCborCodec;
    /// # use multihash::Code::Sha2_256;
    /// # use std::str::FromStr;
    /// # use pretty_assertions::assert_eq;
    /// #
    /// let mut store = MemoryStore::new();
    /// let cid: Cid = FromStr::from_str("bafyreihscx57i276zr5pgnioa5omevods6eseu5h4mllmow6csasju6eqi").unwrap();
    /// let expected = ipld!({"a": 1, "b": {"/": {"link": cid, "data": [1, 2, 3]}}});
    ///
    /// let mut observed = None;
    /// if let Some(Err(mut stuck)) = AtLeastOnce::new(&ipld!({"a": 1, "b": cid})).run(&store) {
    ///   observed = Some(stuck.resolve(ipld!([1, 2, 3]), &mut store).run(&store).expect("A").expect("B"));
    /// }
    ///
    /// /// The IPLD is inlined
    /// assert_eq!(observed.unwrap(), expected);
    ///
    /// // The IPLD is now stored
    /// assert_eq!(store.get(cid).unwrap(), &ipld!([1, 2, 3]));
    /// ```
    pub fn resolve<S: Store>(self, ipld: Ipld, store: &mut S) -> Box<I> {
        store.put_keyed(self.needs, &ipld);
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
    /// #     MemoryStore
    /// #   }
    /// # };
    /// # use libipld::{ipld, Ipld, cid::{CidGeneric, Version}, Cid};
    /// # use libipld::cbor::DagCborCodec;
    /// # use multihash::Code::Sha2_256;
    /// # use pretty_assertions::assert_eq;
    /// # use std::str::FromStr;
    /// #
    /// let mut store = MemoryStore::new();
    /// let cid: Cid = FromStr::from_str("bafyreihscx57i276zr5pgnioa5omevods6eseu5h4mllmow6csasju6eqi").unwrap();
    /// let expected = ipld!({"a": 1, "b": {"/": {"link": cid, "data": [1, 2, 3]}}});
    ///
    /// let mut observed = None;
    /// if let Some(Err(mut stuck)) = AtLeastOnce::new(&ipld!({"a": 1, "b": cid})).run(&store) {
    ///   observed = Some(stuck.stub(ipld!([1, 2, 3])).run(&store).expect("A").expect("B"));
    /// }
    ///
    /// assert_eq!(observed.unwrap(), expected);
    /// ```
    pub fn stub(mut self, ipld: Ipld) -> Box<I> {
        self.inliner
            .deref_mut()
            .resolve(InlineIpld::new(self.needs, ipld).into());

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
    /// #     MemoryStore
    /// #   }
    /// # };
    /// # use libipld::{ipld, Ipld, cid::{CidGeneric, Version}, Cid};
    /// # use libipld::cbor::DagCborCodec;
    /// # use multihash::Code::Sha2_256;
    /// # use std::str::FromStr;
    /// # use pretty_assertions::assert_eq;
    /// #
    /// let mut store = MemoryStore::new();
    /// let cid: Cid = FromStr::from_str("bafyreihscx57i276zr5pgnioa5omevods6eseu5h4mllmow6csasju6eqi").unwrap();
    /// let ipld = ipld!({"a": 1, "b": cid});
    /// let expected = ipld!({"a": 1, "b": cid});
    ///
    /// if let Some(Err(mut stuck)) = AtLeastOnce::new(&ipld).run(&mut store) {
    ///   assert_eq!(stuck.needs(), cid);
    ///   let result = stuck.ignore().run(&store);
    ///   assert_eq!(result.unwrap().unwrap(), expected);
    /// }
    /// ```
    pub fn ignore(mut self) -> Box<I> {
        self.inliner.deref_mut().resolve(Ipld::Link(self.needs));
        self.inliner
    }
}
