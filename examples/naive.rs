use std::time;

use pairwise_aligner::{prelude::*, *};

// Compare with block aligner:
// They do 10k pairs of length 10k and distance 10% in 2s!
fn main() {
    let ns = [250, 500, 1_000, 2_000, 4_000, 8_000, 16_000, 32_000];
    let es = [0.20];

    for n in ns {
        for e in es {
            let (a, b, _, _) = setup(n, e);
            let start = time::Instant::now();
            let dist = bio::alignment::distance::simd::levenshtein(&a, &b);
            let d = start.elapsed().as_secs_f32();
            println!("SIMD {:>8} {:>3.2} {:10.5}s {:>6}", n, e, d, dist);
        }
    }

    println!();

    for n in ns {
        for e in es {
            let (a, b, _, _) = setup(n, e);
            let start = time::Instant::now();
            let dist = bio::alignment::distance::levenshtein(&a, &b);
            let d = start.elapsed().as_secs_f32();
            println!("NAIV {:>8} {:>3.2} {:10.5}s {:>6}", n, e, d, dist);
        }
    }

    println!();

    for n in ns {
        for e in es {
            let (a, b, alph, _) = setup(n, e);
            let start = time::Instant::now();
            let h = GapSeedHeuristic {
                match_config: MatchConfig {
                    length: Fixed(1),
                    ..Default::default()
                },
                pruning: false,
                c: PhantomData::<NaiveContours<LogQueryContour>>,
                ..GapSeedHeuristic::default()
            };
            let h = h.build(&a, &b, &alph);
            let dist = h.h(Node(Pos(0, 0), h.root_state(Pos(0, 0))));
            let d = start.elapsed().as_secs_f32();
            println!("SH   {:>8} {:>3.2} {:10.5}s {:>6}", n, e, d, dist);
        }
    }
}
