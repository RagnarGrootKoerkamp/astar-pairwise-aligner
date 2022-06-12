use super::Aligner;

/// NW aligner for unit costs (Levenshtein distance) only, using library functions.
struct NWLib {
    simd: bool,
}

impl Aligner for NWLib {
    fn cost(
        &self,
        a: &bio_types::sequence::Sequence,
        b: &bio_types::sequence::Sequence,
    ) -> crate::prelude::Cost {
        if self.simd {
            bio::alignment::distance::simd::levenshtein(a, b)
        } else {
            bio::alignment::distance::levenshtein(a, b)
        }
    }

    fn visualize(
        &self,
        _a: &bio_types::sequence::Sequence,
        _b: &bio_types::sequence::Sequence,
        _visualizer: &mut impl super::Visualizer,
    ) -> crate::prelude::Cost {
        unimplemented!("This aligner does not support path tracing!");
    }
}
