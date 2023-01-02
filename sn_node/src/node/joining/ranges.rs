// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use sn_interface::network_knowledge::NetworkKnowledge;
use std::{collections::BTreeSet, ops::RangeInclusive};
use xor_name::XorName;

/// Gets the suitable range for a node to join.
/// This range is the largest available between current members (including to the prefix bounds).
pub(crate) fn get_largest_range(network_knowledge: &NetworkKnowledge) -> RangeInclusive<XorName> {
    let prefix = network_knowledge.prefix();

    // get a sorted set of all xornames, including upper and lower bounds
    let points: BTreeSet<XorName> = network_knowledge
        .members()
        .into_iter()
        .map(|m| m.name())
        .chain([prefix.lower_bound(), prefix.upper_bound()])
        .collect();

    // find the two bounds with largest distance, defaults to the prefix lower and upper bound
    let (start, end) = {
        use itertools::Itertools;
        points
            .into_iter()
            .tuple_windows()
            .max_by(distance::compare)
            .unwrap_or((prefix.lower_bound(), prefix.upper_bound()))
    };

    // // TODO: maybe select a sub-range at some distance from start and end? (as to not get unnecessarily close to another node)

    // return a range from those two bounds
    RangeInclusive::new(start, end)
}

pub(super) mod distance {
    use std::cmp::Ordering;
    use xor_name::{XorName, XOR_NAME_LEN};

    type Range = (XorName, XorName);
    type Distance = [u8; XOR_NAME_LEN];

    // xor comparison
    pub(super) fn compare(a: &Range, b: &Range) -> Ordering {
        let a_dist = distance(a.0, a.1);
        let b_dist = distance(b.0, b.1);
        compare_distances(a_dist, b_dist)
    }

    // xor distance
    fn distance(a: XorName, b: XorName) -> Distance {
        let mut distance = XorName::default().0;
        for i in 0..XOR_NAME_LEN {
            distance[i] = a[i] ^ b[i];
        }
        distance
    }

    // euclidean comparison
    fn compare_distances(a: Distance, b: Distance) -> Ordering {
        for (ai, bi) in a.iter().zip(b.iter()) {
            match ai.cmp(bi) {
                Ordering::Equal => continue,
                ord => return ord,
            }
        }
        Ordering::Equal
    }
}
