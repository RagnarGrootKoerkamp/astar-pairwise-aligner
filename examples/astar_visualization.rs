use pairwise_aligner::{drawing::draw_explored_states, prelude::*};

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
        draw_explored_states(&r, "evals/astar-visualization/dijkstra_transp");
        r.print();
    }
    {
        let h = CSH {
            match_config: MatchConfig {
                length: Fixed(k),
                max_match_cost: m,
                ..MatchConfig::default()
            },
            pruning: false,
            use_gap_cost: false,
            c: PhantomData::<BruteForceContours>::default(),
        };
        let (a, b, alphabet, stats) = setup(n, e);
        let r = align(&a, &b, &alphabet, stats, h);
        r.write_explored_states("evals/astar-visualization/csh-noprune.csv");
        draw_explored_states(&r, "evals/astar-visualization/csh-noprune_transp");
        r.print();
    }
    {
        let h = CSH {
            match_config: MatchConfig {
                length: Fixed(k),
                max_match_cost: m,
                ..MatchConfig::default()
            },
            pruning: true,
            use_gap_cost: false,
            c: PhantomData::<BruteForceContours>::default(),
        };
        let (a, b, alphabet, stats) = setup(n, e);
        let r = align(&a, &b, &alphabet, stats, h);
        r.write_explored_states("evals/astar-visualization/csh.csv");
        draw_explored_states(&r, "evals/astar-visualization/csh_transp");
        r.print();
    }
    {
        let h = SH {
            match_config: MatchConfig {
                length: Fixed(15),
                max_match_cost: 0,
                ..Default::default()
            },
            pruning: false,
        };
        let (a, b, alphabet, stats) = setup(n, e);
        let r = align(&a, &b, &alphabet, stats, h);
        r.write_explored_states("evals/astar-visualization/sh-noprune.csv");
        draw_explored_states(&r, "evals/astar-visualization/sh-noprune_transp");
        r.print();
    }
    {
        let h = SH {
            match_config: MatchConfig {
                length: Fixed(15),
                max_match_cost: 0,
                ..Default::default()
            },
            pruning: true,
        };
        let (a, b, alphabet, stats) = setup(n, e);
        let r = align(&a, &b, &alphabet, stats, h);
        r.write_explored_states("evals/astar-visualization/sh.csv");
        draw_explored_states(&r, "evals/astar-visualization/sh_transp");
        r.print();
    }
}
