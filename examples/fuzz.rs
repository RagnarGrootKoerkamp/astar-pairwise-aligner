#![feature(let_chains)]
use std::panic::AssertUnwindSafe;

use astar_pairwise_aligner::{
    aligners::{astar::AstarPA, Aligner, Sequence},
    prelude::*,
    visualizer::NoVisualizer,
};
use bio::alignment::distance::simd::levenshtein;

fn fuzz(aligner: &mut impl Aligner) -> (Sequence, Sequence) {
    for n in (5..).step_by(1) {
        for r in 0..1000 {
            for e in [0.1, 0.2, 0.4] {
                for m in [
                    ErrorModel::Uniform,
                    ErrorModel::NoisyInsert,
                    ErrorModel::MutatedRepeat,
                ] {
                    println!("n={n} r={r} e={e} m={m:?}");
                    let (ref a, ref b) = generate_model(r, n, e, m);
                    let dist = bio::alignment::distance::simd::levenshtein(&a, &b);
                    let d = std::panic::catch_unwind(AssertUnwindSafe(|| -> Cost {
                        aligner.cost(a, b)
                    }));
                    if let Ok(d) = d && d == dist {
                        continue;
                    }
                    return (a.to_vec(), b.to_vec());
                }
            }
        }
    }
    panic!("No bad cases found");
}

fn main() {
    let dt = true;
    let k = 3;
    let max_match_cost = 1;
    let pruning = true;
    let gap_cost = false;

    let check_dist = true;

    let ref mut aligner = AstarPA {
        dt,
        h: CSH {
            match_config: MatchConfig::new(k, max_match_cost),
            pruning: Pruning::new(pruning),
            use_gap_cost: gap_cost,
            c: PhantomData::<HintContours<BruteForceContour>>::default(),
        },
        v: NoVisualizer,
    };

    // let a = "TCTCTCTCTCTG".as_bytes();
    // let b = "GTCTCTCTTCTG".as_bytes();
    let (ref a, ref b) = fuzz(aligner);
    println!(
        "\n\nShrinking sequences:\nlet a = \"{}\".as_bytes();\nlet b = \"{}\".as_bytes();\n\n",
        to_string(a),
        to_string(b)
    );

    // True on success.
    let mut test = |start: I, end: I| {
        let Pos(n, m) = Pos::target(&a, &b);
        let v = std::panic::catch_unwind(AssertUnwindSafe(|| {
            let a = &a[start as usize..min(n, end) as usize];
            let b = &b[start as usize..min(m, end) as usize];
            let d = aligner.cost(a, b);
            if check_dist {
                let dist = levenshtein(a, b);
                assert_eq!(d, dist);
            }
        }))
        .is_ok();
        println!("Test: {} {} => {}", start, end, v);
        v
    };

    assert!(!test(0, max(a.len(), b.len()) as I));

    let mut start = 0;
    let mut end = max(a.len(), b.len()) as I;

    let mut change = false;

    loop {
        let new_start;
        let new_end;

        // Binary search the start of the sequence in steps of k.
        {
            let mut left = start;
            let mut right = end;
            while left / k < right / k {
                let mid = (left + right + k - 1) / 2 / k * k;
                if test(mid, end) {
                    right = mid - 1;
                } else {
                    left = mid;
                }
            }
            new_start = max(start, left);
        }
        // Binary search the end of the sequence.
        {
            let mut left = start;
            let mut right = end;
            while left < right {
                let mid = (left + right) / 2;
                if test(start, mid) {
                    left = mid + k;
                } else {
                    right = mid;
                }
            }
            new_end = min(end, left);
        }

        if new_start == start && new_end == end {
            start = new_start;
            end = new_end;
            break;
        }
        change = true;
        start = new_start;
        end = new_end;
    }
    if !change {
        assert!(
            !test(start, end),
            "Could not shrink: Testcase doesn't fail!"
        );
    }

    let Pos(n, m) = Pos::target(&a, &b);
    let a = &a[start as usize..min(n, end) as usize].to_vec();
    let b = &b[start as usize..min(m, end) as usize].to_vec();

    println!(
        "\n\nResult of shrinking:\nlet a = \"{}\".as_bytes();\nlet b = \"{}\".as_bytes();\n\n",
        to_string(a),
        to_string(b)
    );
    println!("Aligner:\n{aligner:?}");
}
