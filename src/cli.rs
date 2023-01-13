pub mod heuristic_params;
pub mod input;
pub mod visualizer;

use std::{path::PathBuf, time::Duration};

use crate::cli::{input::Input, visualizer::VisualizerArgs};
use clap::{value_parser, Parser};
use heuristic_params::{AlgorithmArgs, HeuristicArgs};
use serde::{Deserialize, Serialize};

#[derive(Parser, Serialize, Deserialize)]
#[clap(author, about)]
pub struct Cli {
    #[clap(flatten)]
    pub input: Input,

    /// Where to write optional statistics.
    #[arg(short, long, value_parser = value_parser!(PathBuf))]
    pub output: Option<PathBuf>,

    /// Parameters and settings for the algorithm.
    #[clap(flatten)]
    pub algorithm: AlgorithmArgs,

    /// Parameters and settings for the heuristic.
    #[clap(flatten)]
    pub heuristic: HeuristicArgs,

    /// Parameters and settings for the visualizer.
    #[clap(flatten)]
    pub visualizer: VisualizerArgs,

    /// Print less. Pass twice for summary line only.
    ///
    /// Do not print a new line per alignment, but instead overwrite the previous one.
    /// Pass twice to only print a summary line and avoid all terminal clutter, e.g. for benchmarking.
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub silent: u8,

    /// Stop aligning new pairs after this timeout.
    #[arg(long, value_parser = parse_duration::parse , hide_short_help = true)]
    pub timeout: Option<Duration>,
}
