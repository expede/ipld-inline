//! Post-order [`Ipld`] iteration
use core::iter::Peekable;
use libipld::ipld::Ipld;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A post-order [`Ipld`] iterator
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct PostOrderIpldIter<'a> {
    inbound: Vec<&'a Ipld>,
    outbound: Vec<&'a Ipld>,
}

impl<'a> PostOrderIpldIter<'a> {
    /// Initialize a new [`PostOrderIpldIter`]
    pub fn new(ipld: &'a Ipld) -> Self {
        PostOrderIpldIter {
            inbound: vec![ipld],
            outbound: vec![],
        }
    }
}

impl<'a> From<&'a Ipld> for PostOrderIpldIter<'a> {
    fn from(ipld: &'a Ipld) -> Self {
        PostOrderIpldIter::new(ipld)
    }
}

impl<'a> Iterator for PostOrderIpldIter<'a> {
    type Item = &'a Ipld;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inbound.pop() {
                None => return self.outbound.pop(),
                Some(map @ Ipld::Map(btree)) => {
                    self.outbound.push(map);

                    for node in btree.values() {
                        self.inbound.push(node);
                    }
                }

                Some(list @ Ipld::List(vector)) => {
                    self.outbound.push(list);

                    for node in vector {
                        self.inbound.push(node);
                    }
                }
                Some(node) => self.outbound.push(node),
            }
        }
    }
}

/// # Examples
///
/// ```
/// use inline_ipld::iterator::post_order::{PostOrderIpldIter, is_delimiter_next};
/// use libipld::ipld;
/// use std::iter::Peekable;
///
/// let dag = ipld!({"/": 123}); // Will put two items on the stack: [{"/": 123}, 123]
/// let mut poii = PostOrderIpldIter::new(&dag);
/// poii.next(); // Use the lowest item
///
/// assert_eq!(is_delimiter_next(&mut poii.peekable()), true);
/// ```
pub fn is_delimiter_next<'a>(poii: &mut Peekable<PostOrderIpldIter<'a>>) -> bool {
    match poii.peek() {
        Some(Ipld::Map(next_btree)) => next_btree.keys().eq(["/"].iter()),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libipld::{cid::CidGeneric, ipld};
    use pretty_assertions::assert_eq;

    #[test]
    fn poii_test() {
        let cid = Ipld::Link(
            CidGeneric::try_from(
                "bafyreifxzbwbet5pqer5bopvf3wxgvooaijrhynk2wfoksygml6glk44m4".to_string(),
            )
            .unwrap(),
        );

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
            linkless,
            ipld!(99),
            ipld!("hello"),
            linkful_array.clone(),
            cid.clone(),
            ipld!({"link": cid, "data": linkful_array}),
            linkful,
            inlines,
            ipld!("bar-val"),
            ipld!("baz-val"),
            ipld!("foo-val"),
            string_map,
            outer_array,
            ipld.clone(),
        ];

        let mut observed: Vec<Ipld> = vec![];
        for node in PostOrderIpldIter::from(&ipld) {
            observed.push(node.clone());
        }

        assert_eq!(observed, expected);
    }
}
