use std::panic::AssertUnwindSafe;

use pairwise_aligner::prelude::*;

fn main() {
    PRINT.store(false, std::sync::atomic::Ordering::Relaxed);

    let n = 200;
    let e = 0.10;
    let seed = 54;
    let k = 6;
    let max_match_cost = 1;
    let pruning = true;

    let h = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(k),
            max_match_cost,
            ..MatchConfig::default()
        },
        pruning,
        c: PhantomData::<HintContours<BruteForceContour>>,
        ..GapSeedHeuristic::default()
    }
    .equal_to_bruteforce_contours();

    let (a, b, alphabet, stats) = setup_with_seed(n, e, seed);
    println!("Heuristic:\n{:?}", h);

    // True on success.
    let test = |start: I, end: I| {
        let Pos(n, m) = Pos::from_length(&a, &b);
        println!("Test: {} {}", start, end);
        let v = std::panic::catch_unwind(AssertUnwindSafe(|| {
            align(
                &a[start as usize..min(n, end) as usize].to_vec(),
                &b[start as usize..min(m, end) as usize].to_vec(),
                &alphabet,
                stats,
                h,
            )
            .print()
        }))
        .is_ok();
        println!("Test: {} {} => {}", start, end, v);
        v
    };
    let start;
    let mut end = max(a.len(), b.len()) as I;

    // Binary search the start of the sequence in steps of k.
    {
        let mut left = 0;
        let mut right = end;
        while left / k < right / k {
            let mid = (left + right + k - 1) / 2 / k * k;
            if test(mid, end) {
                right = mid - 1;
            } else {
                left = mid;
            }
        }
        start = left;
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
        end = left;
    }

    assert!(
        !test(start, end),
        "Could not shrink: Testcase doesn't fail!"
    );

    let Pos(n, m) = Pos::from_length(&a, &b);
    let a = &a[start as usize..min(n, end) as usize].to_vec();
    let b = &b[start as usize..min(m, end) as usize].to_vec();

    println!("Result\n{}\n{}", to_string(&a), to_string(&b));

    PRINT.store(true, std::sync::atomic::Ordering::Relaxed);
    test(start, end);
    println!("Result\n{}\n{}", to_string(&a), to_string(&b));
}
