use std::panic::AssertUnwindSafe;

use astar_pairwise_aligner::prelude::*;

fn main() {
    PRINT.store(false, std::sync::atomic::Ordering::Relaxed);

    let n = 500;
    let e = 0.3;
    let seed = 31415;
    let k = 6;
    let max_match_cost = 1;
    let pruning = true;

    let h = SH {
        match_config: MatchConfig::new(k, max_match_cost),
        pruning,
    };

    let (a, b, alphabet, stats) = setup_with_seed(n, e, seed);
    println!("Heuristic:\n{:?}", h);

    // True on success.
    let test = |start: I, end: I| {
        let Pos(n, m) = Pos::from_lengths(&a, &b);
        println!("Test: {} {}", start, end);
        let v = std::panic::catch_unwind(AssertUnwindSafe(|| {
            align(
                &a[start as usize..min(n, end) as usize],
                &b[start as usize..min(m, end) as usize],
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
            new_start = left;
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
            new_end = left;
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

    let Pos(n, m) = Pos::from_lengths(&a, &b);
    let a = &a[start as usize..min(n, end) as usize].to_vec();
    let b = &b[start as usize..min(m, end) as usize].to_vec();

    println!("Result\n{}\n{}", to_string(&a), to_string(&b));

    PRINT.store(true, std::sync::atomic::Ordering::Relaxed);
    test(start, end);
    println!("Result\n{}\n{}", to_string(&a), to_string(&b));
}
