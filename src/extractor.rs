use crate::cid::cid_of;
use crate::iterator::post_order::{is_delimiter_next, PostOrderIpldIter};
use core::iter::Peekable;
use libipld::{
    cid,
    cid::Cid,
    codec::{Codec, Encode},
    ipld::Ipld,
};
use multihash::MultihashDigest;

#[derive(Clone, Debug)]
pub struct Extractor<'a, C, D>
where
    D: MultihashDigest<64>,
{
    iterator: Peekable<PostOrderIpldIter>,
    stack: Vec<Ipld>,

    codec: C,
    digester: &'a D,
    cid_version: cid::Version,
}

impl<'a, C: Codec, D: MultihashDigest<64>> Extractor<'a, C, D>
where
    Ipld: Encode<C>,
{
    pub fn new(ipld: Ipld, codec: C, digester: &'a D, cid_version: cid::Version) -> Self {
        Extractor {
            iterator: <Ipld as Into<PostOrderIpldIter>>::into(ipld).peekable(),
            stack: vec![],

            codec,
            digester,
            cid_version,
        }
    }
}

impl<C: Codec, D: MultihashDigest<64>> Iterator for Extractor<'_, C, D>
where
    Ipld: Encode<C>,
{
    type Item = (Cid, Ipld);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iterator.next() {
                None => {
                    return self.stack.pop().map(|x| {
                        (
                            cid_of(&x, self.codec, self.digester, self.cid_version).unwrap(),
                            x,
                        )
                    });
                }
                Some(Ipld::List(inner_list)) => {
                    let substack = self.stack.split_off(self.stack.len() - inner_list.len());
                    self.stack.push(Ipld::List(substack));
                }
                Some(Ipld::Map(btree)) => {
                    let keys: Vec<&String> = btree.keys().collect();

                    if btree.get("data").is_some() {
                        if keys.len() == 1 && is_delimiter_next(&mut self.iterator) {
                            self.iterator.next(); // i.e. skip delimiter

                            let node = self
                                .stack
                                .pop()
                                .expect("updated child node of 'data' should be on the stack");

                            let cid: Cid =
                                cid_of(&node, self.codec, self.digester, self.cid_version).unwrap();

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

                    self.stack
                        .push(Ipld::Map(keys.into_iter().cloned().zip(substack).collect()));
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

    use libipld::{cid::CidGeneric, ipld};
    use libipld_cbor::DagCborCodec;
    use multihash::Code::Sha2_256;
    use proptest::prelude::*;
    use std::collections::BTreeMap;

    #[derive(Clone, Debug, PartialEq)]
    struct MoreThanIpld(Ipld);

    impl Arbitrary for MoreThanIpld {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            prop_oneof![
                Just(Ipld::Null),
                any::<bool>().prop_map(Ipld::Bool),
                any::<Vec<u8>>().prop_map(Ipld::Bytes),
                any::<i64>().prop_map(|i| { Ipld::Integer(i as i128) }), // because we're testing the DagCborCodec, and Ipld isn't type safe on `Ipld::Integer`s
                any::<f64>().prop_map(Ipld::Float),
                ".*".prop_map(Ipld::String),
                any::<u32>().prop_map(|i| {
                    let encoded = DagCborCodec.encode(&Ipld::Integer(i.into())).unwrap();
                    let multihash = Sha2_256.digest(encoded.as_slice());
                    let cid = Cid::new_v1(Sha2_256.into(), multihash);
                    Ipld::Link(cid)
                })
            ]
            .prop_recursive(8, 256, 64, |inner| {
                prop_oneof![
                    prop::collection::vec(inner.clone(), 0..64).prop_map(Ipld::List),
                    prop::collection::btree_map(".*", inner, 0..64).prop_map(Ipld::Map),
                ]
            })
            .prop_map(MoreThanIpld)
            .boxed()
        }
    }

    proptest! {
        #[test]
        fn identity_prop_test(MoreThanIpld(ipld) in any::<MoreThanIpld>()) {
            let mut ext = Extractor::new(&ipld, DagCborCodec, Sha2_256, cid::Version::V1);
            prop_assert!(ext.next().unwrap().1 == ipld);
        }
    }

    #[test]
    fn store_identity_test() {
        let cid = CidGeneric::try_from(
            "bafyreie5xtjxubxwtytnuymfknf6ivzagr3grsj6bwf57lqohgydct3ite".to_string(),
        )
        .unwrap();

        let ipld =
            ipld!({"a": ["b", 1, 2, {"c": "d"}], "e": {"/": {"data": 123, "don't match": 42}}});

        let mut expected: BTreeMap<Cid, Ipld> = BTreeMap::new();
        expected.insert(cid, ipld.clone());

        let mut observed: BTreeMap<Cid, Ipld> = BTreeMap::new();
        for (cid, node) in Extractor::new(&ipld, DagCborCodec, Sha2_256, cid::Version::V1) {
            observed.insert(cid, node);
        }

        assert_eq!(observed, expected);
    }

    #[test]
    fn store_single_top_test() {
        let ipld = ipld!({"/": {"data": [1, 2, 3]}});

        let mut observed: BTreeMap<Cid, Ipld> = BTreeMap::new();
        for (cid, node) in Extractor::new(&ipld, DagCborCodec, Sha2_256, cid::Version::V1) {
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

        let ipld = ipld!({"/": {"data": [1, 2, 3], "link": arr_cid}});

        let mut observed: BTreeMap<Cid, Ipld> = BTreeMap::new();
        for (cid, node) in Extractor::new(&ipld, DagCborCodec, Sha2_256, cid::Version::V1) {
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
        for (cid, node) in Extractor::new(&ipld, DagCborCodec, Sha2_256, cid::Version::V1) {
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

        let mut observed: BTreeMap<Cid, Ipld> = BTreeMap::new();
        for (cid, node) in Extractor::new(&ipld, DagCborCodec, Sha2_256, cid::Version::V1) {
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
        for (cid, node) in Extractor::new(&ipld, DagCborCodec, Sha2_256, cid::Version::V1) {
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

        let mut observed: BTreeMap<Cid, Ipld> = BTreeMap::new();
        for (cid, node) in Extractor::new(&ipld, DagCborCodec, Sha2_256, cid::Version::V1) {
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

        let mut observed: BTreeMap<Cid, Ipld> = BTreeMap::new();
        for (cid, node) in Extractor::new(&ipld, DagCborCodec, Sha2_256, cid::Version::V1) {
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
