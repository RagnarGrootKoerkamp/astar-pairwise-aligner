use pairwise_aligner::{prelude::*, *};

fn main() {
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path("evals/stats/table.csv")
        .unwrap();

    let n = 300;
    let e = 0.10;
    let l = 3;
    let max_match_cost = 0;
    let pruning = true;

    let result = {
        let h = GapSeedHeuristic {
            match_config: MatchConfig {
                length: Fixed(l),
                max_match_cost,
                ..MatchConfig::default()
            },
            pruning,
            prune_fraction: 0.5,
            c: PhantomData::<NaiveContours<NaiveContour>>,
            ..GapSeedHeuristic::default()
        };
        let (a, b, alphabet, stats) = setup(n, e);
        align(&a, &b, &alphabet, stats, h)
    };
    result.print();
    result.write(&mut wtr);
}
