use crate::{
    aligners::{nw_lib::NWLib, Aligner},
    cli::heuristic_params::Algorithm,
    prelude::*,
    runner::{AlignWithHeuristic, Cli},
};
use wasm_bindgen::prelude::*;
use std::ops::ControlFlow;

#[wasm_bindgen]
pub fn run() {
    let args: Cli = serde_json::from_str("abc").unwrap();
    args.input.process_input_pairs(|a: Seq, b: Seq| {
        // Run the pair.
        // TODO: Show the result somewhere.
        let _r = if args.algorithm.algorithm.external() {
            let start = instant::Instant::now();
            let cost = match args.algorithm.algorithm {
                Algorithm::NwLib => NWLib { simd: false }.cost(a, b),
                Algorithm::NwLibSimd => NWLib { simd: true }.cost(a, b),
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
        ControlFlow::Break(())
    });
}
