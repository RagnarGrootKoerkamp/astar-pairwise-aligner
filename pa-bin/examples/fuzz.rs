#![feature(let_chains)]
use astarpa::AstarPa;
use bio::alignment::distance::simd::levenshtein;
use pa_generate::{generate_model, uniform_fixed, ErrorModel};
use pa_heuristic::{Heuristic, MatchConfig, Prune, Pruning, CSH};
use pa_types::{seq_to_string, Cost, Pos, Sequence, I};
use pa_vis::visualizer::{self, When};
use pa_vis_types::{NoVis, VisualizerT};
use std::{
    cmp::{max, min},
    panic::AssertUnwindSafe,
};

#[allow(unused)]
fn fuzz<V: VisualizerT, H: Heuristic>(aligner: &AstarPa<V, H>) -> (Sequence, Sequence) {
    for n in (5..).step_by(1) {
        for r in 0..1000 {
            for e in [0.1, 0.2, 0.4] {
                for m in [
                    ErrorModel::Uniform,
                    ErrorModel::NoisyInsert,
                    ErrorModel::SymmetricRepeat,
                ] {
                    println!("n={n} r={r} e={e} m={m:?}");
                    let (ref a, ref b) = generate_model(n, e, m, r);
                    let dist = bio::alignment::distance::simd::levenshtein(&a, &b) as Cost;
                    let d = std::panic::catch_unwind(AssertUnwindSafe(|| -> Cost {
                        aligner.align(a, b).0 .0
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
    let k = 4;
    let max_match_cost = 0;
    let pruning = Prune::Both;

    let check_dist = true;

    let h = CSH::new(MatchConfig::new(k, max_match_cost), Pruning::new(pruning));
    let ref mut aligner = AstarPa {
        dt,
        h: h.equal_to_bruteforce_csh(),
        v: NoVis,
    };

    // let a = "TCTCTCTCTCTG".as_bytes();
    // let b = "GTCTCTCTTCTG".as_bytes();

    //let (ref a, ref b) = fuzz(aligner);

    let (ref a, ref b) = uniform_fixed(40, 0.3);

    println!(
        "\n\nShrinking sequences:\nlet a = \"{}\".as_bytes();\nlet b = \"{}\".as_bytes();\n\n",
        seq_to_string(a),
        seq_to_string(b)
    );

    // True on success.
    let test = |start: I, end: I| {
        let Pos(n, m) = Pos::target(&a, &b);
        let v = std::panic::catch_unwind(AssertUnwindSafe(|| {
            let a = &a[start as usize..min(n, end) as usize];
            let b = &b[start as usize..min(m, end) as usize];
            let d = aligner.align(a, b).0 .0;
            if check_dist {
                let dist = levenshtein(a, b) as _;
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
        seq_to_string(a),
        seq_to_string(b)
    );
    println!("Aligner:\n{aligner:?}");

    let mut aligner = AstarPa {
        dt: aligner.dt,
        h: CSH::new(MatchConfig::new(k, max_match_cost), Pruning::new(pruning)).to_bruteforce_csh(),
        v: visualizer::Config::new(visualizer::VisualizerStyle::Debug),
    };
    aligner.v.draw = When::All;
    aligner.align(a, b);
}
