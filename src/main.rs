#![feature(let_chains)]

use astar_pairwise_aligner::{
    aligners::{triple_accel::TripleAccel, Aligner},
    cli::heuristic_params::Algorithm,
    prelude::*,
    runner::{AlignWithHeuristic, Cli},
};
use clap::Parser;
use itertools::Itertools;
use std::{ops::ControlFlow, time::Instant};

fn main() {
    let args = Cli::parse();
    //println!("{}", serde_json::to_string_pretty(&args).unwrap());

    // Read the input
    let mut avg_result = AlignResult::default();
    let start = instant::Instant::now();

    args.input.process_input_pairs(|a: Seq, b: Seq| {
        // Run the pair.
        let r = if args.algorithm.algorithm.external() {
            let start = Instant::now();
            let cost = match args.algorithm.algorithm {
                Algorithm::TripleAccel => TripleAccel {
                    exp_search: args.algorithm.exp_search,
                }
                .cost(a, b),
                Algorithm::Edlib => {
                    #[cfg(not(feature = "edlib"))]
                    panic!("Enable the edlib feature flag to use edlib.");
                    #[cfg(feature = "edlib")]
                    aligners::edlib::Edlib.cost(a, b)
                }
                Algorithm::Wfa => {
                    #[cfg(not(feature = "wfa"))]
                    panic!("Enable the wfa feature flag to use WFA.");
                    #[cfg(feature = "wfa")]
                    aligners::wfa::WFA {
                        cm: LinearCost::new_unit(),
                        biwfa: false,
                    }
                    .cost(a, b)
                }
                Algorithm::Biwfa => {
                    #[cfg(not(feature = "wfa"))]
                    panic!("Enable the wfa feature flag to use BiWFA.");
                    #[cfg(feature = "wfa")]
                    aligners::wfa::WFA {
                        cm: LinearCost::new_unit(),
                        biwfa: true,
                    }
                    .cost(a, b)
                }
                _ => unreachable!(),
            };
            let total_duration = start.elapsed().as_secs_f32();
            AlignResult::new(a, b, cost, total_duration)
        } else {
            args.heuristic
                .run_on_heuristic(AlignWithHeuristic { a, b, args: &args })
        };

        // Record and print stats.
        avg_result.add_sample(&r);
        if args.silent <= 1 {
            print!("\r");
            if args.silent == 0 {
                r.print();
            }
            avg_result.print_no_newline();
        }

        if let Some(d) = args.timeout && start.elapsed() > d {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    });

    if avg_result.sample_size > 0 {
        print!("\r");
        avg_result.print();

        if let Some(output) = args.output {
            let (header, vals) = avg_result.values();

            std::fs::write(
                output,
                format!(
                    "{}\n{}\n",
                    header.iter().map(|x| x.trim()).join("\t"),
                    vals.iter().map(|x| x.trim()).join("\t")
                ),
            )
            .unwrap();
        }
    }
}

#[cfg(test)]
mod test {
    use super::Cli;

    #[test]
    fn cli_test() {
        <Cli as clap::CommandFactory>::command().debug_assert();
    }
}
