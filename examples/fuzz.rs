#![feature(let_chains)]
use std::panic::AssertUnwindSafe;

use astar_pairwise_aligner::prelude::*;

fn main() {
    for n in (48..).step_by(1) {
        for r in 0..10000 {
            for e in [0.1, 0.2, 0.4] {
                let (k, m, n, e, pruning) = (4, 1, n, e, true);
                let h = CSH {
                    match_config: MatchConfig::new(k, m),
                    pruning: Pruning {
                        enabled: pruning,
                        skip_prune: 0,
                    },
                    use_gap_cost: true,
                    c: PhantomData::<HintContours<BruteForceContour>>::default(),
                };

                println!("n={n} r={r} k={k} m={m} e={e}");
                let (ref a, ref b) =
                    setup_sequences_with_seed_and_model(r, n, e, ErrorModel::NoisyDelete);
                let stats = InputStats {
                    len_a: a.len(),
                    len_b: b.len(),
                    error_rate: e,
                };

                let dist = bio::alignment::distance::simd::levenshtein(&a, &b);
                let result = std::panic::catch_unwind(AssertUnwindSafe(|| align(&a, &b, stats, h)));
                if let Ok(result) = result && result.edit_distance == dist {
                    continue;
                }
                panic!(
                    "\nlet a = \"{}\".as_bytes();\nlet b = \"{}\".as_bytes();\n",
                    to_string(a),
                    to_string(b)
                );
            }
        }
    }
}
