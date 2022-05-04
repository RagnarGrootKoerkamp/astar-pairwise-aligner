use pairwise_aligner::prelude::*;

fn main() {
    let n = 500;
    let e = 0.2;

    let m = 1;
    let k = 9;

    // {
    //     let h = ZeroCost;
    //     let (a, b, alphabet, stats) = setup(n, e);
    //     let r = align(&a, &b, &alphabet, stats, h);
    //     r.write_explored_states("evals/astar-visualization/dijkstra.csv");
    //     r.print();
    // }
    {
        let h = ZeroCost;
        let (a, b, alphabet, stats) = setup(n, e);
        let r = align_advanced(&a, &b, &alphabet, stats, h, false);
        r.write_explored_states("evals/astar-visualization/dijkstra-nogreedy.csv");
        r.print();
    }
    {
        let h = SH {
            match_config: MatchConfig {
                length: Fixed(k),
                max_match_cost: m,
                ..MatchConfig::default()
            },
            pruning: false,
        };
        let (a, b, alphabet, stats) = setup(n, e);
        let r = align_advanced(&a, &b, &alphabet, stats, h, false);
        r.write_explored_states("evals/astar-visualization/sh-noprune.csv");
        r.print();
    }
    {
        let h = SH {
            match_config: MatchConfig {
                length: Fixed(k),
                max_match_cost: m,
                ..MatchConfig::default()
            },
            pruning: true,
        };
        let (a, b, alphabet, stats) = setup(n, e);
        let r = align(&a, &b, &alphabet, stats, h);
        r.write_explored_states("evals/astar-visualization/sh.csv");
        r.print();
    }
}
