use super::{cigar::Cigar, nw::PATH, Aligner};

/// NW aligner for unit costs (Levenshtein distance) only, using library functions.
pub struct NWLib {
    pub simd: bool,
}

impl Aligner for NWLib {
    fn cost(
        &self,
        a: &bio_types::sequence::Sequence,
        b: &bio_types::sequence::Sequence,
    ) -> crate::prelude::Cost {
        if self.simd {
            // TODO: Note that this actually uses exponential search as well.
            bio::alignment::distance::simd::levenshtein(a, b)
        } else {
            bio::alignment::distance::levenshtein(a, b)
        }
    }

    fn visualize(
        &self,
        _a: &bio_types::sequence::Sequence,
        _b: &bio_types::sequence::Sequence,
        _visualizer: &mut impl super::VisualizerT,
    ) -> (crate::prelude::Cost, PATH, Cigar) {
        unimplemented!("NWLib does not support path tracing.");
    }
}
