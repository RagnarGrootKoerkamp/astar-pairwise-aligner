#![feature(let_chains)]
//~ This file is mostly identical to `pa-bin/src/main.rs`, but wraps the given
// heuristic in the `PathHeuristic`. To achieve this, some more functions are inlined here.

use astarpa::HeuristicParams;
use astarpa_next::path_pruning::PathHeuristic;
use clap::Parser;
use pa_affine_types::{AffineAligner, AffineCost};
use pa_base_algos::{
    nw::{AffineFront, NW},
    Domain,
};
use pa_bin::Cli;
use pa_heuristic::{Heuristic, HeuristicMapper};
use pa_types::*;
use pa_vis::{NoVis, VisualizerT};
use std::{
    io::{BufWriter, Write},
    ops::ControlFlow,
};

pub fn astar_aligner() -> Box<dyn AffineAligner> {
    make_path_heuristic_aligner(NoVis)
}

fn make_path_heuristic_aligner(vis: impl VisualizerT + 'static) -> Box<dyn AffineAligner> {
    let dt = true;
    let h = &HeuristicParams::default();
    struct Mapper<V: VisualizerT> {
        #[allow(unused)]
        dt: bool,
        v: V,
    }
    impl<V: VisualizerT + 'static> HeuristicMapper for Mapper<V> {
        type R = Box<dyn AffineAligner>;
        fn call<H: Heuristic + 'static>(self, h: H) -> Box<dyn AffineAligner> {
            Box::new(NW {
                cm: AffineCost::unit(),
                strategy: pa_base_algos::Strategy::LocalDoubling,
                domain: Domain::Astar(PathHeuristic { h }),
                block_width: 1,
                v: self.v,
                front: AffineFront,
                trace: true,
                sparse_h: true,
                prune: true,
            })
        }
    }

    h.map(Mapper { dt, v: vis })
}

fn main() {
    let args = Cli::parse();

    let mut aligner = astar_aligner();

    let mut out_file = args
        .output
        .as_ref()
        .map(|o| BufWriter::new(std::fs::File::create(o).unwrap()));

    // Process the input.
    args.process_input_pairs(|a: Seq, b: Seq| {
        // Run the pair.
        let (cost, cigar) = aligner.align_affine(a, b);

        if let Some(f) = &mut out_file {
            writeln!(f, "{cost},{}", cigar.unwrap().to_string()).unwrap();
        }
        ControlFlow::Continue(())
    });
}

#[cfg(test)]
mod test {
    #[test]
    fn cli_test() {
        <super::Cli as clap::CommandFactory>::command().debug_assert();
    }
}
