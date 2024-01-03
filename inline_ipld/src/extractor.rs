//! Strategies for decomposing inlined [`Ipld`] to a DAG
//!
//! This is typically called via [`Store::extract`][crate::store::Store::extract]

use crate::{
    cid,
    codec::EncodableAs,
    iterator::{is_delimiter_next, PostOrderIpldIter},
    InlineIpld,
};
use core::iter::Peekable;
use libipld::{
    cid::{Cid, Version},
    codec::{Codec, Encode},
    ipld::Ipld,
};
use multihash::MultihashDigest;
use std::collections::btree_map::{BTreeMap, Keys};

/// The general [`Ipld`] extraction strategy
///
/// Converts Inline IPLD into "regular" [`Ipld`]. This does a series of graph nodes.
#[derive(Clone, Debug)]
pub struct Extractor<'a, C, D>
where
    D: MultihashDigest<64>,
{
    iterator: Peekable<PostOrderIpldIter<'a>>,
    stack: Vec<Ipld>,

    codec: C,
    digester: &'a D,
    cid_version: Version,
}

impl<'a, C: Codec, D: MultihashDigest<64>> Extractor<'a, C, D>
where
    Ipld: Encode<C>,
{
    /// Initialize an [`Extractor`]
    ///
    /// # Arguments
    ///
    /// * `ipld` - The inline [`Ipld`] to extract
    /// * `codec` - The [`Codec`] to fall back to if the inline IPLD doesn't contain a [`Cid`]
    /// * `digester` - The hash digest function to use if the inline IPLD doesn't contain a [`Cid`]
    /// * `cid_version` - The [`Cid`] version to use if the inline IPLD doesn't contain a [`Cid`]
    pub fn new(
        inline_ipld: &'a InlineIpld,
        codec: C,
        digester: &'a D,
        cid_version: Version,
    ) -> Self {
        Extractor {
            iterator: PostOrderIpldIter::new(inline_ipld.into()).peekable(),
            stack: vec![],
            codec,
            digester,
            cid_version,
        }
    }
}

impl<'a, C: Codec, D: MultihashDigest<64>> Iterator for Extractor<'a, C, D>
where
    Ipld: EncodableAs<C>,
{
    type Item = (Cid, Ipld);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iterator.next() {
                None => {
                    return self
                        .stack
                        .pop()
                        .map(|x| (cid::new(&x, self.codec, self.digester, self.cid_version), x));
                }

                Some(Ipld::List(inner_list)) => {
                    let substack = self.stack.split_off(self.stack.len() - inner_list.len());
                    self.stack.push(Ipld::List(substack));
                }

                Some(Ipld::Map(btree)) => {
                    let keys: Keys<'_, String, Ipld> = btree.keys();

                    if btree.get("data").is_some() {
                        if keys.len() == 1 && is_delimiter_next(&mut self.iterator) {
                            self.iterator.next(); // i.e. skip delimiter

                            let node = self
                                .stack
                                .pop()
                                .expect("updated child node of 'data' should be on the stack"); // FIXME

                            let cid = cid::new(&node, self.codec, self.digester, self.cid_version);

                            self.stack.push(Ipld::Link(cid));
                            return Some((cid, node));
                        }

                        if keys.len() == 2 && is_delimiter_next(&mut self.iterator) {
                            if let Some(Ipld::Link(_)) = btree.get("link") {
                                self.iterator.next(); // i.e. skip delimiter

                                if let Some(ref link @ Ipld::Link(cid)) = self.stack.pop() {
                                    let node = self.stack.pop().expect(
                                        "updated child node of 'data' should be on the stack",
                                    );

                                    self.stack.push(link.clone());
                                    return Some((cid, node));
                                }

                                panic!("An Ipld::Link should be on the stack")
                            }
                        }
                    }

                    let substack: Vec<Ipld> = self.stack.split_off(self.stack.len() - keys.len());
                    let inner_map: BTreeMap<String, Ipld> =
                        keys.zip(substack).map(|(k, v)| (k.clone(), v)).collect();

                    self.stack.push(Ipld::Map(inner_map));
                }

                Some(node) => {
                    self.stack.push(node.clone());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        test_util::{cid_config::CidConfig, super_ipld::SuperIpld},
        InlineIpld,
    };
    use libipld::cbor::DagCborCodec;
    use libipld::{cid::CidGeneric, ipld};
    use multihash::Code::Sha2_256;
    use pretty_assertions::assert_eq;
    use proptest::prelude::*;
    use std::collections::BTreeMap;

    // FIXME more props!
    proptest! {
        #[test]
        fn identity_ipld_prop_test((SuperIpld(ipld), CidConfig{ digester, version, codec }) in (any::<SuperIpld>(), any::<CidConfig>())) {
            let inline = InlineIpld::attest(ipld.clone());
            let mut ext = Extractor::new(&inline, codec, &digester, version);
            prop_assert_eq!(ext.next().unwrap().1, ipld);
        }

        #[test]
        fn correct_cid_prop_test((SuperIpld(ipld), CidConfig{ digester, version, codec }) in (any::<SuperIpld>(), any::<CidConfig>())) {
            let inline = InlineIpld::attest(ipld);
            for (cid, dag) in Extractor::new(&inline, codec, &digester, version) {
              prop_assert_eq!(cid, cid::new(&dag, codec, &digester, version));
            }
        }
    }

    #[test]
    fn store_identity_test() {
        let cid = CidGeneric::try_from(
            "bafyreie5xtjxubxwtytnuymfknf6ivzagr3grsj6bwf57lqohgydct3ite".to_string(),
        )
        .unwrap();

        let ipld = ipld!({
            "a": ["b", 1, 2, {"c": "d"}],
            "e": {"/": {"data": 123, "don't match": 42}}
        });

        let mut expected: BTreeMap<Cid, Ipld> = BTreeMap::new();
        expected.insert(cid, ipld.clone());

        let mut observed: BTreeMap<Cid, Ipld> = BTreeMap::new();
        let inline = InlineIpld::attest(ipld);
        for (cid, node) in Extractor::new(&inline, DagCborCodec, &Sha2_256, Version::V1) {
            observed.insert(cid, node);
        }

        assert_eq!(observed, expected);
    }

    #[test]
    fn store_single_top_test() {
        let arr_cid: Cid = CidGeneric::try_from(
            "bafyreickxqyrg7hhhdm2z24kduovd4k4vvbmfmenzn7nc6pxg6qzjm2v44".to_string(),
        )
        .unwrap();

        let inline = InlineIpld::new(arr_cid, ipld!([1, 2, 3]));

        let mut observed: BTreeMap<Cid, Ipld> = BTreeMap::new();
        for (cid, node) in Extractor::new(&inline, DagCborCodec, &Sha2_256, Version::V1) {
            observed.insert(cid, node);
        }

        let cid1: Cid = CidGeneric::try_from(
            "bafyreickxqyrg7hhhdm2z24kduovd4k4vvbmfmenzn7nc6pxg6qzjm2v44".to_string(),
        )
        .unwrap();

        let cid2: Cid = CidGeneric::try_from(
            "bafyreihnubkcms63243zlfgnwiugmk6ijitz63me7bqf455ia2fpbn4ceq".to_string(),
        )
        .unwrap();

        let mut expected = BTreeMap::new();
        expected.insert(cid1, ipld!([1, 2, 3]));
        expected.insert(cid2, ipld!(cid1));

        assert_eq!(observed, expected);
    }

    #[test]
    fn store_single_top_linkful_test() {
        let arr_cid: Cid = CidGeneric::try_from(
            "bafyreickxqyrg7hhhdm2z24kduovd4k4vvbmfmenzn7nc6pxg6qzjm2v44".to_string(),
        )
        .unwrap();

        let outer_cid: Cid = CidGeneric::try_from(
            "bafyreihnubkcms63243zlfgnwiugmk6ijitz63me7bqf455ia2fpbn4ceq".to_string(),
        )
        .unwrap();

        let inline = InlineIpld::new(arr_cid, ipld!([1, 2, 3]));

        let mut observed: BTreeMap<Cid, Ipld> = BTreeMap::new();
        for (cid, node) in Extractor::new(&inline, DagCborCodec, &Sha2_256, Version::V1) {
            observed.insert(cid, node);
        }

        let mut expected = BTreeMap::new();
        expected.insert(arr_cid, ipld!([1, 2, 3]));
        expected.insert(outer_cid, ipld!(arr_cid));

        assert_eq!(observed, expected);
    }

    #[test]
    fn store_single_not_top_test() {
        let ipld = ipld!([{"/": {"data": [1, 2, 3]}}]);

        let mut observed: BTreeMap<Cid, Ipld> = BTreeMap::new();
        let inline = InlineIpld::attest(ipld);
        for (cid, node) in Extractor::new(&inline, DagCborCodec, &Sha2_256, Version::V1) {
            observed.insert(cid, node);
        }

        let cid1: Cid = CidGeneric::try_from(
            "bafyreickxqyrg7hhhdm2z24kduovd4k4vvbmfmenzn7nc6pxg6qzjm2v44".to_string(),
        )
        .unwrap();

        let cid2: Cid = CidGeneric::try_from(
            "bafyreic6rlmkazpohhul74xyu654gs4k37idb2uz6r7vurebasdi766kga".to_string(),
        )
        .unwrap();

        let mut expected = BTreeMap::new();
        expected.insert(cid1, ipld!([1, 2, 3]));
        expected.insert(cid2, ipld!([cid1]));

        assert_eq!(observed, expected);
    }

    #[test]
    fn store_single_not_top_linkful_test() {
        let arr_cid: Cid = CidGeneric::try_from(
            "bafyreickxqyrg7hhhdm2z24kduovd4k4vvbmfmenzn7nc6pxg6qzjm2v44".to_string(),
        )
        .unwrap();

        let outer_cid: Cid = CidGeneric::try_from(
            "bafyreic6rlmkazpohhul74xyu654gs4k37idb2uz6r7vurebasdi766kga".to_string(),
        )
        .unwrap();

        let ipld = ipld!([{"/": {"data": [1, 2, 3], "link": arr_cid}}]);
        let inline = InlineIpld::attest(ipld);

        let mut observed: BTreeMap<Cid, Ipld> = BTreeMap::new();
        for (cid, node) in Extractor::new(&inline, DagCborCodec, &Sha2_256, Version::V1) {
            observed.insert(cid, node);
        }

        let mut expected = BTreeMap::new();
        expected.insert(arr_cid, ipld!([1, 2, 3]));
        expected.insert(outer_cid, ipld!([arr_cid]));

        assert_eq!(observed, expected);
    }

    #[test]
    fn store_nested_test() {
        let ipld = ipld!({"/": {"data": [1, {"/": {"data": ["a", "b"]}}]}});
        let inline = InlineIpld::attest(ipld);

        let mut expected: BTreeMap<Cid, Ipld> = BTreeMap::new();

        let cid1: Cid = CidGeneric::try_from(
            "bafyreia5h7xzw5e2wknxfzd5qmty3ebe452q7iwys6qo6lstpi5mlknkyu".to_string(),
        )
        .unwrap();

        expected.insert(cid1, ipld!(["a", "b"]));

        let cid2: Cid = CidGeneric::try_from(
            "bafyreieytegtxlityotbbwbe3445s327jghqlbwyv7k7kxnpzjj7k3c6yu".to_string(),
        )
        .unwrap();

        expected.insert(cid2, ipld!([1, cid1]));

        let cid3: Cid = CidGeneric::try_from(
            "bafyreifxzbwbet5pqer5bopvf3wxgvooaijrhynk2wfoksygml6glk44m4".to_string(),
        )
        .unwrap();

        expected.insert(cid3, ipld!(cid2));

        let mut observed: BTreeMap<Cid, Ipld> = BTreeMap::new();
        for (cid, node) in Extractor::new(&inline, DagCborCodec, &Sha2_256, Version::V1) {
            observed.insert(cid, node);
        }

        assert_eq!(observed, expected);
    }

    #[test]
    fn store_nested_linkful_test() {
        let inner_cid: Cid = CidGeneric::try_from(
            "bafyreia5h7xzw5e2wknxfzd5qmty3ebe452q7iwys6qo6lstpi5mlknkyu".to_string(),
        )
        .unwrap();

        let mid_cid: Cid = CidGeneric::try_from(
            "bafyreieytegtxlityotbbwbe3445s327jghqlbwyv7k7kxnpzjj7k3c6yu".to_string(),
        )
        .unwrap();

        let outer_cid: Cid = CidGeneric::try_from(
            "bafyreifxzbwbet5pqer5bopvf3wxgvooaijrhynk2wfoksygml6glk44m4".to_string(),
        )
        .unwrap();

        let ipld = ipld!(
            {
                "/": {
                    "link": mid_cid,
                    "data": [
                        1,
                        {
                            "/": {
                                "link": inner_cid,
                                "data": ["a", "b"]
                            }
                        }
                    ]
                }
            }
        );
        let inline = InlineIpld::attest(ipld);

        let mut observed: BTreeMap<Cid, Ipld> = BTreeMap::new();
        for (cid, node) in Extractor::new(&inline, DagCborCodec, &Sha2_256, Version::V1) {
            observed.insert(cid, node);
        }

        let mut expected: BTreeMap<Cid, Ipld> = BTreeMap::new();
        expected.insert(inner_cid, ipld![["a", "b"]]);
        expected.insert(mid_cid, ipld![[1, inner_cid]]);
        expected.insert(outer_cid, ipld![mid_cid]);

        assert_eq!(observed, expected);
    }

    #[test]
    fn store_mixed_test() {
        let arr_cid: Cid = CidGeneric::try_from(
            "bafyreia5h7xzw5e2wknxfzd5qmty3ebe452q7iwys6qo6lstpi5mlknkyu".to_string(),
        )
        .unwrap();

        let mid_cid: Cid = CidGeneric::try_from(
            "bafyreifxzbwbet5pqer5bopvf3wxgvooaijrhynk2wfoksygml6glk44m4".to_string(),
        )
        .unwrap();

        let entry_cid: Cid = CidGeneric::try_from(
            "bafyreihxkjjf3kxhwiozngod4zlbhwzqqybn2f6fm5lot7xfobjiuxg63m".to_string(),
        )
        .unwrap();

        let outer_cid: Cid = CidGeneric::try_from(
            "bafyreibvo5xlmuj5jluhvsrl57goinrvcojh4c3n2k2z7fwido3pyxrct4".to_string(),
        )
        .unwrap();

        let ipld = ipld!(
            {
                "entry":{
                    "/": {
                        "data": [
                            1,
                            {"/": {"link": arr_cid, "data": ["a", "b"]}},
                            2,
                            3
                        ]
                    }
                },
                "more": ["hello", "world"],
                "don't match": {
                    "/": {
                        "data": [4, 5, 6],
                        "breaks!": "NOPE!",
                        "do match": {
                            "/": {
                                "link": mid_cid,
                                "data": [7, 8, 9]
                            }
                        }
                    }
                }
            }
        );
        let inline = InlineIpld::attest(ipld);

        let mut observed: BTreeMap<Cid, Ipld> = BTreeMap::new();
        for (cid, node) in Extractor::new(&inline, DagCborCodec, &Sha2_256, Version::V1) {
            observed.insert(cid, node);
        }

        let mut expected: BTreeMap<Cid, Ipld> = BTreeMap::new();
        expected.insert(arr_cid, ipld!(["a", "b"]));
        expected.insert(mid_cid, ipld!([7, 8, 9]));
        expected.insert(entry_cid, ipld!([1, arr_cid, 2, 3]));
        expected.insert(
            outer_cid,
            ipld!({
                "entry": entry_cid,
                "more": ["hello", "world"],
                "don't match": {"/": {"breaks!": "NOPE!", "data": [4, 5, 6], "do match": mid_cid}},
            }),
        );

        assert_eq!(observed, expected);
    }
}
