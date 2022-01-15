use pairwise_aligner::prelude::*;

fn main() {
    let n = 100;
    let e = 0.30;
    let l = 4;
    let max_match_cost = 0;
    let pruning = true;
    let prune_fraction = 1.0;

    let h = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(l),
            max_match_cost,
            ..MatchConfig::default()
        },
        pruning,
        prune_fraction,
        c: PhantomData::<NaiveContours<LogQueryContour>>,
        ..GapSeedHeuristic::default()
    }
    .equal_to_seed_heuristic();

    let (a, b, alphabet, stats) = setup(n, e);
    println!("Heuristic:\n{:?}", h);

    // True on success.
    let test = |start: I, end: I| {
        let Pos(n, m) = Pos::from_length(&a, &b);
        println!("Test: {} {}", start, end);
        let v = std::panic::catch_unwind(|| {
            align(
                &a[start as usize..min(n, end) as usize].to_vec(),
                &b[start as usize..min(m, end) as usize].to_vec(),
                &alphabet,
                stats,
                h,
            )
            .print()
        })
        .is_ok();
        println!("Test: {} {} => {}", start, end, v);
        v
    };
    let start;
    let mut end = max(a.len(), b.len()) as I;

    // Binary search the start of the sequence in steps of l.
    {
        let mut left = 0;
        let mut right = end;
        while left / l < right / l {
            let mid = (left + right) / 2 / l * l;
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
                left = mid + l;
            } else {
                right = mid;
            }
        }
        end = left;
    }
    assert!(!test(start, end));
    println!("Result\n{}\n{}", to_string(&a), to_string(&b));
}
