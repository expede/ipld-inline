use core::iter::Peekable;
use libipld::codec::{Codec, Encode};
use libipld::{cid::Cid, ipld::Ipld};
use multihash::Code::Sha2_256;
use multihash::MultihashDigest;
// use std::collections::BTreeMap;
use std::marker::PhantomData;

// FIXME: unwraps & clones
// FIXME: CidGeneric

mod iterator;

#[derive(Debug, PartialEq, Clone)]
pub struct PostOrderIpldIter<'a> {
    inbound: Vec<&'a Ipld>,  // work
    outbound: Vec<&'a Ipld>, // stash
}

impl<'a> From<&'a Ipld> for PostOrderIpldIter<'a> {
    fn from(ipld_ref: &'a Ipld) -> Self {
        PostOrderIpldIter {
            inbound: vec![ipld_ref],
            outbound: vec![],
        }
    }
}

impl<'a> Iterator for PostOrderIpldIter<'a> {
    type Item = &'a Ipld;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inbound.pop() {
                None => return self.outbound.pop(),
                Some(map @ Ipld::Map(btree)) => {
                    self.outbound.push(&map);

                    for node in btree.values() {
                        self.inbound.push(node);
                    }
                }

                Some(list @ Ipld::List(vector)) => {
                    self.outbound.push(&list);

                    for node in vector.iter() {
                        self.inbound.push(node);
                    }
                }
                Some(node) => self.outbound.push(node),
            }
        }
    }
}

#[test]
fn poii_test() {
    use libipld::ipld;
    use libipld::multihash::MultihashDigest;

    let multihash = Sha2_256.digest([1, 2, 3].as_slice()); // FIXME coded on those bytes

    let cid = ipld!(Cid::new_v1(0, multihash));
    let linkless_array = ipld!(["world", 123, 456]);
    let linkful_array = ipld!([99, "hello"]);
    let string_map = ipld!({"bar": "bar-val", "baz": "baz-val", "foo": "foo-val"});
    let linkless = ipld!({"/": {"data": linkless_array.clone()}});
    let linkful = ipld!({"/": {"link": cid.clone(), "data": linkful_array.clone()}});
    let inlines = ipld!({
        "computes the cid": linkless.clone(),
        "uses existing cid": linkful.clone()
    });
    let outer_array = ipld!([inlines.clone(), string_map.clone()]);
    let ipld = ipld!({"Here goes": outer_array.clone()});

    let expected: Vec<Ipld> = vec![
        ipld!("world"),
        ipld!(123),
        ipld!(456),
        linkless_array.clone(),
        ipld!({"data": linkless_array}),
        linkless.clone(),
        ipld!(99),
        ipld!("hello"),
        linkful_array.clone(),
        cid.clone(),
        ipld!({"link": cid, "data": linkful_array}),
        linkful.clone(),
        inlines.clone(),
        ipld!("bar-val"),
        ipld!("baz-val"),
        ipld!("foo-val"),
        string_map.clone(),
        outer_array.clone(),
        ipld.clone(),
    ];

    let mut observed: Vec<Ipld> = vec![];
    for node in PostOrderIpldIter::from(&ipld) {
        observed.push(node.clone());
    }

    assert_eq!(observed, expected);
}

#[derive(Clone, Debug)]
pub struct TheBreakerUpper<'a, C> {
    phantom_codec: PhantomData<C>,
    stack: Vec<Ipld>,
    iterator: Peekable<PostOrderIpldIter<'a>>,
}

impl<'a, C: Codec + Default> TheBreakerUpper<'a, C>
where
    Ipld: Encode<C>,
{
    fn new(ipld: &'a Ipld, _codec: C) -> Self {
        TheBreakerUpper {
            phantom_codec: PhantomData as PhantomData<C>,
            stack: vec![],
            iterator: <&Ipld as Into<PostOrderIpldIter>>::into(ipld).peekable(),
        }
    }
}

impl<'a, C: Codec + Default> From<(&'a Ipld, C)> for TheBreakerUpper<'a, C>
where
    Ipld: Encode<C>,
{
    fn from(i: (&'a Ipld, C)) -> Self {
        TheBreakerUpper::new(i.0, i.1)
    }
}

impl<C: Codec + Default> Iterator for TheBreakerUpper<'_, C>
where
    Ipld: Encode<C>,
{
    type Item = (Cid, Ipld);

    fn next(&mut self) -> Option<Self::Item> {
        let codec: C = Default::default();
        // FIXME maybe a dirty flag?

        loop {
            match self.iterator.next() {
                None => match self.stack.pop() {
                    None => return None,
                    Some(x) => {
                        dbg!(self.stack.clone());
                        return Some((cid_of(&x, codec), x));
                    }
                },
                Some(Ipld::List(inner_list)) => {
                    let substack = self.stack.split_off(self.stack.len() - inner_list.len());
                    self.stack.push(Ipld::List(substack));
                }
                Some(Ipld::Map(btree)) => {
                    let keys: Vec<String> = btree.keys().map(|x| x.clone()).collect();

                    if let Some(_) = btree.get("data") {
                        if keys.len() == 1 && is_delimiter_next(&mut self.iterator) {
                            self.iterator.next(); // i.e. skip delimiter

                            let node = self.stack.pop().unwrap();
                            let cid: Cid = cid_of(&node, codec);

                            self.stack.push(Ipld::Link(cid));
                            return Some((cid, node));
                        }

                        if keys.len() == 2 && is_delimiter_next(&mut self.iterator) {
                            if let Some(Ipld::Link(_)) = btree.get("link") {
                                self.iterator.next(); // i.e. skip delimiter

                                match self.stack.pop() {
                                    Some(ref link @ Ipld::Link(cid)) => {
                                        let node = self.stack.pop().unwrap();

                                        self.stack.push(link.clone());
                                        return Some((cid, node));
                                    }
                                    _ => panic!("expected an Ipld::Link"),
                                }
                            }
                        }
                    }

                    let substack = self.stack.split_off(self.stack.len() - keys.len());

                    self.stack.push(Ipld::Map(
                        keys.iter().map(|x| x.clone()).zip(substack).collect(),
                    ));
                }

                Some(node) => {
                    self.stack.push(node.clone());
                }
            }
        }
    }
}

/// # Examples
///
/// ```
/// use ipld_inline::{PostOrderIpldIter, is_delimiter_next};
/// use libipld::ipld;
/// use std::iter::Peekable;
///
/// let dag = ipld!({"/": 123}); // Will put two items on the stack: [{"/": 123}, 123]
/// let mut poii: PostOrderIpldIter = (&dag).into();
/// poii.next(); // Use the lowest item
///
/// assert_eq!(is_delimiter_next(&mut poii.peekable()), true);
/// ```
pub fn is_delimiter_next(poii: &mut Peekable<PostOrderIpldIter>) -> bool {
    match poii.peek() {
        Some(Ipld::Map(next_btree)) => next_btree.keys().collect::<Vec<&String>>() == vec![&"/"],
        _ => false,
    }
}

#[test]
fn store_identity_test() {
    use libipld::cid::CidGeneric;
    use libipld::ipld;
    use libipld_cbor::DagCborCodec;
    use std::collections::BTreeMap;

    let cid = CidGeneric::try_from(
        "bafyreie5xtjxubxwtytnuymfknf6ivzagr3grsj6bwf57lqohgydct3ite".to_string(),
    )
    .unwrap();

    let ipld = ipld!({"a": ["b", 1, 2, {"c": "d"}], "e": {"/": {"data": 123, "don't match": 42}}});

    let mut expected: BTreeMap<Cid, Ipld> = BTreeMap::new();
    expected.insert(cid, ipld.clone());

    let mut observed: BTreeMap<Cid, Ipld> = BTreeMap::new();
    for (cid, node) in TheBreakerUpper::new(&ipld, DagCborCodec) {
        observed.insert(cid, node);
    }

    assert_eq!(observed, expected);
}

#[test]
fn store_single_top_test() {
    use libipld::cid::CidGeneric;
    use libipld::ipld;
    use libipld_cbor::DagCborCodec;
    use std::collections::BTreeMap;

    let ipld = ipld!({"/": {"data": [1, 2, 3]}});

    let mut observed: BTreeMap<Cid, Ipld> = BTreeMap::new();
    for (cid, node) in TheBreakerUpper::new(&ipld, DagCborCodec) {
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
    use libipld::cid::CidGeneric;
    use libipld::ipld;
    use libipld_cbor::DagCborCodec;
    use std::collections::BTreeMap;

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
    for (cid, node) in TheBreakerUpper::new(&ipld, DagCborCodec) {
        observed.insert(cid, node);
    }

    let mut expected = BTreeMap::new();
    expected.insert(arr_cid, ipld!([1, 2, 3]));
    expected.insert(outer_cid, ipld!(arr_cid));

    assert_eq!(observed, expected);
}

#[test]
fn store_single_not_top_test() {
    use libipld::cid::CidGeneric;
    use libipld::ipld;
    use libipld_cbor::DagCborCodec;
    use std::collections::BTreeMap;

    let ipld = ipld!([{"/": {"data": [1, 2, 3]}}]);

    let mut observed: BTreeMap<Cid, Ipld> = BTreeMap::new();
    for (cid, node) in TheBreakerUpper::new(&ipld, DagCborCodec) {
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
    use libipld::cid::CidGeneric;
    use libipld::ipld;
    use libipld_cbor::DagCborCodec;
    use std::collections::BTreeMap;

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
    for (cid, node) in TheBreakerUpper::new(&ipld, DagCborCodec) {
        observed.insert(cid, node);
    }

    let mut expected = BTreeMap::new();
    expected.insert(arr_cid, ipld!([1, 2, 3]));
    expected.insert(outer_cid, ipld!([arr_cid]));

    assert_eq!(observed, expected);
}

#[test]
fn store_nested_test() {
    use libipld::cid::CidGeneric;
    use libipld::ipld;
    use libipld_cbor::DagCborCodec;
    use std::collections::BTreeMap;

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
    for (cid, node) in TheBreakerUpper::new(&ipld, DagCborCodec) {
        observed.insert(cid, node);
    }

    assert_eq!(observed, expected);
}

#[test]
fn store_nested_linkful_test() {
    use libipld::cid::CidGeneric;
    use libipld::ipld;
    use libipld_cbor::DagCborCodec;
    use std::collections::BTreeMap;

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
    for (cid, node) in TheBreakerUpper::new(&ipld, DagCborCodec) {
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
    use libipld::cid::CidGeneric;
    use libipld::ipld;
    use libipld_cbor::DagCborCodec;
    use std::collections::BTreeMap;

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
    for (cid, node) in TheBreakerUpper::new(&ipld, DagCborCodec) {
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

fn cid_of<C: Codec>(ipld: &Ipld, codec: C) -> Cid
where
    Ipld: Encode<C>,
{
    let encoded = codec.encode(ipld).unwrap();
    let multihash = Sha2_256.digest(encoded.as_slice()); // FIXME coded on those bytes
    Cid::new_v1(codec.into(), multihash)
}
