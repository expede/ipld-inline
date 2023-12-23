use crate::store::traits::Store;
use libipld::{ipld, Cid, Ipld};

pub trait Inliner<'a> {
    // FIXME
    //     /// The primary interface for [`Inliner`]s
    //     ///
    //     /// # Examples
    //     ///
    //     /// ```
    //     /// # use ipld_inline::{
    //     /// #   inliner::exactly_once::ExactlyOnce,
    //     /// #   store::{
    //     /// #     traits::Store,
    //     /// #     memory::MemoryStore
    //     /// #   }
    //     /// # };
    //     /// # use libipld::{ipld, cid::{CidGeneric, Version}};
    //     /// # use libipld_cbor::DagCborCodec;
    //     /// # use multihash::Code::Sha2_256;
    //     /// #
    //     /// let mut store = MemoryStore::new();
    //     /// let cid = store.put(ipld!([1, 2, 3]), DagCborCodec, &Sha2_256, Version::V1).unwrap();
    //     ///
    //     /// let mut exactly_once = ExactlyOnce::new(ipld!({"a": 1, "b": cid}), &mut store);
    //     /// let expected = ipld!({"a": 1, "b": {"/": {"link": cid, "data": [1, 2, 3]}}});
    //     ///
    //     /// assert_eq!(exactly_once.run().unwrap().unwrap(), expected);
    //     /// ```
    //     /// FIXME the above can't compare in the eq
    //     /// FIXME show the err case
    fn run(&'a mut self) -> Result<&Ipld, Stuck<'a, Self>>;

    // These are shared with Stuck... should that get extracted?
    fn store(&mut self, cid: &Cid, ipld: &Ipld);
    fn stub(&mut self, ipld: &Ipld) -> ();
    fn skip(&'a mut self) -> &'a mut Self; // FIXME
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
    pub fn stub(&'a mut self, ipld: &Ipld) -> &'a mut I {
        self.inliner.stub(
            &ipld!({ // FIXME break out a "inline chunk" helper. Maybe just `inline!`?
                "/": {
                    "data": ipld.clone(),
                    "link": self.needs.clone()
                }
            }),
        );

        self.inliner.skip() // TODO needs = None;
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
    pub fn ignore(&'a mut self) -> &'a mut I {
        self.inliner.stub(&Ipld::Link(self.needs.clone()));
        self.inliner.skip()
    }
}
