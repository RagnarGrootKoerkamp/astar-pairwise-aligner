use pairwise_aligner::prelude::*;

fn main() {
    let n = 70;
    let e = 0.3;

    let m = 0;
    let k = 4;

    let (ref a, ref b, ref alphabet, stats) = setup_with_seed(n, e, 1);

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
    for pruning in [false, true] {
        {
            let h = SH {
                match_config: MatchConfig {
                    length: Fixed(k),
                    max_match_cost: m,
                    ..MatchConfig::default()
                },
                pruning,
            };
            let mut h = Heuristic::build(&h, &a, &b, &alphabet);
            let graph = EditGraph::new(a, b, true);
            let (distance_and_path, astar) = astar::astar(&graph, Pos(0, 0), &mut h);
            let (_distance, path) = distance_and_path.unwrap_or_default();

            // TODO: Add matches
            h.display(
                target,
                hmax,
                Some(astar.explored_states),
                Some(astar.expanded_states),
                Some(path),
            );
        }
        {
            let h = CSH {
                match_config: MatchConfig {
                    length: Fixed(k),
                    max_match_cost: m,
                    ..MatchConfig::default()
                },
                pruning,
                use_gap_cost: false,
                c: PhantomData::<HintContours<BruteForceContour>>::default(),
            };
            let mut h = Heuristic::build(&h, &a, &b, &alphabet);
            let graph = EditGraph::new(a, b, true);
            let (distance_and_path, astar) = astar::astar(&graph, Pos(0, 0), &mut h);
            let (_distance, path) = distance_and_path.unwrap_or_default();

            h.display(
                target,
                hmax,
                Some(astar.explored_states),
                Some(astar.expanded_states),
                Some(path),
            );
        }
    }
}
