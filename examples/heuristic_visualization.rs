use pairwise_aligner::prelude::*;

fn main() {
    let n = 100;
    let e = 0.2;

    let m = 0;
    let k = 4;

    let (ref a, ref b, ref alphabet, stats) = setup(n, e);

    let target = Pos::from_length(&a, &b);
    let hmax = Some(align(a, b, alphabet, stats, ZeroCost).edit_distance);

    // {
    //     let h = ZeroCost;
    //     let h = Heuristic::build(&h, &a, &b, &alphabet);
    //     h.display(target, DisplayType::Heuristic, hmax);
    // }
    // {
    //     let h = PerfectHeuristic;
    //     let h = Heuristic::build(&h, &a, &b, &alphabet);
    //     h.display(target, DisplayType::Heuristic, hmax);
    // }
    // {
    //     let h = GapCost;
    //     let h = Heuristic::build(&h, &a, &b, &alphabet);
    //     h.display(target, DisplayType::Heuristic, hmax);
    // }
    // {
    //     let h = CountCost;
    //     let h = Heuristic::build(&h, &a, &b, &alphabet);
    //     h.display(target, DisplayType::Heuristic, hmax);
    // }
    let d = DisplayOptions {};
    // {
    //     let h = SH {
    //         match_config: MatchConfig {
    //             length: Fixed(k),
    //             max_match_cost: m,
    //             ..MatchConfig::default()
    //         },
    //         pruning: false,
    //     };
    //     let h = Heuristic::build(&h, &a, &b, &alphabet);
    //     h.display(target, d, hmax);
    // }
    {
        let h = CSH {
            match_config: MatchConfig {
                length: Fixed(k),
                max_match_cost: m,
                ..MatchConfig::default()
            },
            pruning: true,
            use_gap_cost: false,
            c: PhantomData::<HintContours<BruteForceContour>>::default(),
        };
        let mut h = Heuristic::build(&h, &a, &b, &alphabet);
        //h.display(a, b, target, d, hmax, None, None, None);
        let graph = AlignmentGraph::new(a, b, true);
        let (distance_and_path, astar) = astar::astar(&graph, Pos(0, 0), &mut h);
        let (_distance, path) = distance_and_path.unwrap_or_default();

        h.display(
            a,
            b,
            target,
            d,
            hmax,
            Some(astar.explored_states),
            Some(astar.expanded_states),
            Some(path),
        );
    }
}
