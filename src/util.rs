// Common types reexported.

pub use bio::{
    alphabets::{Alphabet, RankTransform},
    data_structures::qgram_index::QGramIndex,
};
pub use bio_types::sequence::Sequence;
pub use std::cmp::{max, min};
pub use std::collections::BTreeMap;

pub fn abs_diff(i: usize, j: usize) -> usize {
    (i as isize - j as isize).abs() as usize
}

#[derive(Debug, PartialEq, Eq)]
pub struct Mutations {
    pub deletions: Vec<usize>,
    pub substitutions: Vec<usize>,
    pub insertions: Vec<usize>,
}

// TODO: Do not generate insertions at the end. (Also do not generate similar
// sequences by inserting elsewhere.)
// TODO: Move to seeds.rs.
pub fn mutations(k: usize, kmer: usize) -> Mutations {
    // This assumes the alphabet size is 4.
    let mut deletions = Vec::with_capacity(k);
    let mut substitutions = Vec::with_capacity(4 * k);
    let mut insertions = Vec::with_capacity(4 * (k + 1));
    // Substitutions
    for i in 0..k {
        let mask = !(3 << (2 * i));
        for s in 0..4 {
            // TODO: Skip the identity substitution.
            substitutions.push((kmer & mask) | s << (2 * i));
        }
    }
    // Insertions
    // TODO: Test that excluding insertions at the start and end doesn't matter.
    for i in 0..=k {
        let mask = (1 << (2 * i)) - 1;
        for s in 0..4 {
            insertions.push((kmer & mask) | (s << (2 * i)) | ((kmer & !mask) << 2));
        }
    }
    // Deletions
    for i in 0..k {
        let mask = (1 << (2 * i)) - 1;
        deletions.push((kmer & mask) | ((kmer & (!mask << 2)) >> 2));
    }
    for v in [&mut deletions, &mut substitutions, &mut insertions] {
        // TODO: This sorting is slow; maybe we can work around it.
        v.sort_unstable();
        v.dedup();
    }
    // Remove original
    substitutions.retain(|&x| x != kmer);
    Mutations {
        deletions,
        substitutions,
        insertions,
    }
}

pub fn to_string(seq: &[u8]) -> String {
    String::from_utf8(seq.to_vec()).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_mutations() {
        let kmer = 0b00011011usize;
        let k = 4;
        let ms = mutations(k, kmer);
        // substitution
        assert!(ms.substitutions.contains(&0b11011011));
        // insertion
        assert!(ms.insertions.contains(&0b0011011011));
        // deletion
        assert!(ms.deletions.contains(&0b000111));
        assert_eq!(
            ms,
            Mutations {
                deletions: [6, 7, 11, 27].to_vec(),
                substitutions: [11, 19, 23, 24, 25, 26, 31, 43, 59, 91, 155, 219].to_vec(),
                insertions: [
                    27, 75, 91, 99, 103, 107, 108, 109, 110, 111, 123, 155, 219, 283, 539, 795
                ]
                .to_vec(),
            }
        );
    }

    #[test]
    fn kmer_removal() {
        let kmer = 0b00011011usize;
        let k = 4;
        let ms = mutations(k, kmer);
        assert!(!ms.substitutions.contains(&kmer));
        assert!(ms.deletions.contains(&kmer));
        assert!(ms.insertions.contains(&kmer));
    }
}
