//! Given a pattern, find all occurrences of the pattern in the text.
//! Basically, compute the full semi-global alignment table and return the bottom row of costs.
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

use std::{
    cmp::min,
    fs::File,
    io::BufReader,
    marker::PhantomData,
    path::PathBuf,
    time::{Duration, Instant},
};

use astarpa::{stats::AstarStats, AstarPa};
use bio::io::fasta;
use clap::{value_parser, Parser};
use itertools::Itertools;
use pa_bitpacking::{BitProfile, HEncoding, Profile, ScatterProfile, V};
use pa_heuristic::{
    contour::{sorted_contour::SortedContour, HintContours},
    DefaultCSH, MatchConfig, Prune, Pruning, CSH,
};
use pa_vis::{
    canvas::{BLACK, RED, WHITE},
    visualizer::{self, Gradient, When},
    NoVis,
};

#[derive(Parser)]
pub struct Cli {
    #[clap(short, long, value_parser = value_parser!(PathBuf), display_order = 1)]
    pub patterns: PathBuf,
    #[clap(short, long, value_parser = value_parser!(PathBuf), display_order = 1)]
    pub texts: PathBuf,

    #[clap(long)]
    pub astar: bool,
    #[clap(long)]
    pub dt: bool,

    #[clap(short, long)]
    pub scat: bool,

    #[clap(short, long, default_value_t = 1.0)]
    pub u: f32,

    #[clap(long)]
    pub rc: bool,

    #[clap(short, default_value_t = 15)]
    pub k: usize,

    #[clap(long, default_value_t = 3)]
    pub lp: usize,
}

fn main() {
    let args = Cli::parse();

    let mut texts = fasta::Reader::new(BufReader::new(File::open(&args.texts).unwrap()))
        .records()
        .map(|x| x.unwrap());
    let patterns = fasta::Reader::new(BufReader::new(File::open(&args.patterns).unwrap()))
        .records()
        .map(|x| x.unwrap())
        .collect_vec();

    let start = Instant::now();

    let mut all_stats = AstarStats::default();

    for (_i, text) in texts.by_ref().take(1).enumerate() {
        for (j, pattern) in patterns.iter().enumerate() {
            eprintln!("\n\n PATTERN {j}\n");
            let seq = pattern.seq();

            if args.astar {
                let k = args.k as _;
                let mut match_config = MatchConfig::exact(k);
                match_config.local_pruning = args.lp;

                let pruning1 = Pruning::new(Prune::None);

                let h = CSH {
                    match_config,
                    pruning: pruning1,
                    use_gap_cost: true,
                    c: PhantomData::<HintContours<SortedContour>>,
                };
                let ((cost, _cigar), stats) = AstarPa {
                    dt: args.dt,
                    h,
                    v: NoVis,
                }
                .align(text.seq(), seq);
                eprintln!("Cost {cost}");
                eprintln!("stats {stats:?}");
                all_stats += stats;
            } else {
                let result = pa_bitpacking::search::search(seq, text.seq(), args.u);
                // println!("{:?}", result.out);

                // let rc = bio::alphabets::dna::revcomp(pattern.seq());
                // seq = &rc;
                // let start = std::time::Instant::now();
                // let result = pa_bitpacking::search::search(seq, text.seq(), args.u);
                eprintln!("search {:?}", start.elapsed());
                // println!("{:?}", result.out);

                // Do a trace back from the minimum index.
                let idx = result.out.iter().position_min().unwrap();
                eprintln!(
                    "idx {idx} pos {} has value {}",
                    result.idx_to_pos(idx),
                    result.out[idx],
                );
                let start = std::time::Instant::now();
                let (cigar, path) = result.trace(idx);
                eprintln!("trace {:?}", start.elapsed());
                eprintln!("From {} to {}", path.first().unwrap(), path.last().unwrap());
                // eprintln!("{cigar:?}");
                // eprintln!("{path:?}");
            }
        }
    }
    eprintln!("ALL STATS\n{all_stats:?}");
    assert_eq!(
        texts.next(),
        None,
        "Only one text/reference is currently supported."
    );
    eprintln!("Duration: {:?}", start.elapsed());
}
