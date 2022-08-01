use astar_pairwise_aligner::prelude::*;

fn main() {
    let n = 2000;
    let e = 0.20;
    let k = 7;
    let max_match_cost = 1;

    let h_fast = CSH {
        match_config: MatchConfig::new(k, max_match_cost),
        pruning: Pruning::enabled(),
        use_gap_cost: true,
        c: PhantomData::<HintContours<BruteForceContour>>,
    };
    let h_base = CSH {
        match_config: MatchConfig::new(k, max_match_cost),
        pruning: Pruning::enabled(),
        use_gap_cost: true,
        c: PhantomData::<BruteForceContours>,
    };
    let h = EqualHeuristic {
        h1: h_base,
        h2: h_fast,
    };

    let (a, b, alphabet, stats) = setup(n, e);
    let start = 910;
    let end = 1050;
    let a = (&a[start..end - 20]).to_vec();
    let b = (&b[start + 20..end]).to_vec();
    let result = align(&a, &b, &alphabet, stats, h);
    result.print();
}
