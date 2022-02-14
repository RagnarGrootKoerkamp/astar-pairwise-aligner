use pairwise_aligner::prelude::*;

fn main() {
    let n = 500;
    let e = 0.3;

    {
        let h = ZeroCost;
        let (a, b, alphabet, stats) = setup(n, e);
        let r = align(&a, &b, &alphabet, stats, h);
        r.write_explored_states("evals/astar-visualization/dijkstra.csv");
        r.print();
    }
    {
        let h = GapSeedHeuristic {
            match_config: MatchConfig {
                length: Fixed(6),
                max_match_cost: 1,
                ..MatchConfig::default()
            },
            pruning: false,
            c: PhantomData::<HintContours<BruteForceContour>>,
            ..GapSeedHeuristic::default()
        };
        let (a, b, alphabet, stats) = setup(n, e);
        let r = align(&a, &b, &alphabet, stats, h);
        r.write_explored_states("evals/astar-visualization/astar_no_pruning.csv");
        r.print();
    }
    {
        let h = GapSeedHeuristic {
            match_config: MatchConfig {
                length: Fixed(6),
                max_match_cost: 1,
                ..MatchConfig::default()
            },
            pruning: true,
            prune_fraction: 1.0,
            c: PhantomData::<HintContours<BruteForceContour>>,
            ..GapSeedHeuristic::default()
        };
        let (a, b, alphabet, stats) = setup(n, e);
        let r = align(&a, &b, &alphabet, stats, h);
        r.write_explored_states("evals/astar-visualization/astar_pruning.csv");
        r.print();
    }
}
