#[cfg(not(feature = "sdl2"))]
fn main() {}

#[cfg(feature = "sdl2")]
fn main() {
    use astar_pairwise_aligner::prelude::*;

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
        let h = CSH {
            match_config: MatchConfig::new(k, m),
            pruning: false,
            use_gap_cost: false,
            c: PhantomData::<BruteForceContours>::default(),
        };
        let (a, b, alphabet, stats) = setup(n, e);
        let r = align(&a, &b, &alphabet, stats, h);
        r.write_explored_states("evals/astar-visualization/csh-noprune.csv");
        r.print();
    }
    {
        let h = CSH {
            match_config: MatchConfig::new(k, m),
            pruning: true,
            use_gap_cost: false,
            c: PhantomData::<BruteForceContours>::default(),
        };
        let (a, b, alphabet, stats) = setup(n, e);
        let r = align(&a, &b, &alphabet, stats, h);
        r.write_explored_states("evals/astar-visualization/csh.csv");
        r.print();
    }
}
