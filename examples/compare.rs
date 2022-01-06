use itertools::Itertools;
use pairwise_aligner::{prelude::*, *};

fn main() {
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path("evals/stats/table.csv")
        .unwrap();

    let ns = [8_000, 16_000, 32_000, 64_000, 128_000];
    let es = [0.20];
    let lm = [
        (Fixed(4), 0),
        (Fixed(5), 0),
        (Fixed(6), 0),
        (Fixed(7), 0),
        (LengthConfig::max(0, |n| (n as f32).sqrt() as usize), 0),
        (LengthConfig::max(1, |n| (n as f32).sqrt() as usize), 0),
        (LengthConfig::min(1, |n| (n as f32).sqrt() as usize), 0),
        (LengthConfig::min(2, |n| (n as f32).sqrt() as usize), 0),
        (LengthConfig::max(0, |n| n), 0),
        (LengthConfig::max(1, |n| n), 0),
        (LengthConfig::min(1, |n| n), 0),
        (LengthConfig::min(2, |n| n), 0),
        (Fixed(6), 1),
        (Fixed(7), 1),
        (Fixed(8), 1),
        (Fixed(9), 1),
        (Fixed(10), 1),
        (LengthConfig::max(0, |n| (n as f32).sqrt() as usize), 1),
        (LengthConfig::max(1, |n| (n as f32).sqrt() as usize), 1),
        (LengthConfig::min(1, |n| (n as f32).sqrt() as usize), 1),
        (LengthConfig::min(2, |n| (n as f32).sqrt() as usize), 1),
        (LengthConfig::max(0, |n| n), 1),
        (LengthConfig::max(1, |n| n), 1),
        (LengthConfig::min(1, |n| n), 1),
        (LengthConfig::min(2, |n| n), 1),
    ];
    let prunings = [false];
    let build_fast = [(true, QueryMode::On)];

    for (&n, e) in ns.iter().cartesian_product(es) {
        for (length, max_match_cost) in lm {
            for pruning in prunings {
                for (build_fast, query_fast) in build_fast {
                    if pruning && query_fast.enabled() {
                        continue;
                    }
                    if !pruning && !query_fast.enabled() {
                        continue;
                    }
                    let result = {
                        let h = SeedHeuristic {
                            match_config: MatchConfig {
                                length,
                                max_match_cost,
                                ..MatchConfig::default()
                            },
                            distance_function: GapHeuristic,
                            pruning,
                            build_fast,
                            query_fast,
                        };
                        let (a, b, alphabet, stats) = setup(n, e);
                        align(&a, &b, &alphabet, stats, h)
                    };
                    result.print();
                    result.write(&mut wtr);
                }
            }
        }
        println!("");
    }
}
