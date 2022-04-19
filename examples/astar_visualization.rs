use pairwise_aligner::prelude::*;

fn main() {
    let n = 500;
    let e = 0.2;

    let m = 1;
    let k = 9;

    {
        let h = ZeroCost;
        let (a, b, alphabet, stats) = setup(n, e);
        let r = align(&a, &b, &alphabet, stats, h);
        r.write_explored_states("evals/astar-visualization/dijkstra.csv");
        r.print();
    }
    {
        let h = ZeroCost;
        let (a, b, alphabet, stats) = setup(n, e);
        let r = align_advanced(&a, &b, &alphabet, stats, h, false);
        r.write_explored_states("evals/astar-visualization/dijkstra_nogreedy.csv");
        r.print();
    }
    {
        let h = GapSeedHeuristic {
            match_config: MatchConfig {
                length: Fixed(k),
                max_match_cost: m,
                ..MatchConfig::default()
            },
            pruning: false,
            c: PhantomData::<HintContours<BruteForceContour>>,
        };
        let (a, b, alphabet, stats) = setup(n, e);
        let r = align_advanced(&a, &b, &alphabet, stats, h, false);
        r.write_explored_states("evals/astar-visualization/astar_no_pruning.csv");
        r.print();
    }
    {
        let h = GapSeedHeuristic {
            match_config: MatchConfig {
                length: Fixed(k),
                max_match_cost: m,
                ..MatchConfig::default()
            },
            pruning: true,
            c: PhantomData::<HintContours<BruteForceContour>>,
        };
        let (a, b, alphabet, stats) = setup(n, e);
        let r = align(&a, &b, &alphabet, stats, h);
        r.write_explored_states("evals/astar-visualization/astar_pruning.csv");
        r.print();
    }
}
