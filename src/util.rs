// Common types reexported.

pub use bio::{
    alphabets::{Alphabet, RankTransform},
    data_structures::qgram_index::QGramIndex,
};
pub use bio_types::sequence::Sequence;
pub use std::cmp::{max, min};
pub use std::collections::BTreeMap;
pub use std::collections::HashMap;

use serde::Serialize;
use std::cmp::Ordering;

// A position in a pairwise matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct Pos(pub usize, pub usize);

/// Partial ordering by (a,b) < (c,d) when a<c and b<d.
impl PartialOrd for Pos {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let a = self.0.partial_cmp(&other.0);
        let b = self.1.partial_cmp(&other.1);
        if a == b {
            return a;
        }
        if a == Some(Ordering::Equal) {
            return b;
        }
        if b == Some(Ordering::Equal) {
            return a;
        }
        return None;
    }
}

pub fn abs_diff(i: usize, j: usize) -> usize {
    (i as isize - j as isize).abs() as usize
}
