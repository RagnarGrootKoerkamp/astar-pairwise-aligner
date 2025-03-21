//! Given a pattern, find all occurrences of the pattern in the text.
//! Basically, compute the full semi-global alignment table and return the bottom row of costs.
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

use std::{
    cmp::max,
    collections::HashMap,
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
use log::info;
use pa_heuristic::{
    contour::{sorted_contour::SortedContour, HintContours},
    MatchConfig, Prune, Pruning, CSH,
};
use pa_types::{Pos, I};
use pa_vis::NoVis;
use packed_seq::{PackedSeqVec, Seq, SeqVec};
use smallvec::SmallVec;

#[derive(Parser)]
pub struct Cli {
    #[clap(value_parser = value_parser!(PathBuf))]
    pub texts: PathBuf,
    #[clap(value_parser = value_parser!(PathBuf))]
    pub patterns: PathBuf,

    #[clap(long)]
    pub map: bool,

    #[clap(long)]
    pub astar: bool,

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
        if args.map {
            let patterns = patterns.iter().map(|p| p.seq()).collect_vec();
            map(text.seq(), &patterns, args.k as I);
            continue;
        }
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
                    dt: true,
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
                let start = Instant::now();
                let (_cigar, path) = result.trace(idx);
                eprintln!("trace {:?}", start.elapsed());
                eprintln!("From {} to {}", path.first().unwrap(), path.last().unwrap());
                // eprintln!("{cigar:?}");
                // eprintln!("{path:?}");
            }
        }
    }
    if !args.map {
        eprintln!("ALL STATS\n{all_stats:?}");
    }
    assert_eq!(
        texts.next(),
        None,
        "Only one text/reference is currently supported."
    );
    eprintln!("Duration: {:?}", start.elapsed());
}

#[derive(Clone, Copy, Debug)]
struct TPos(I, I);

const MIN_MATCH_FRACTION: f32 = 0.25;

fn map(text: &[u8], patterns: &[&[u8]], k: I) {
    let n = text.len() as I;

    let mut t = Timer::new();
    let mut s = Stats::new();

    // 1. Build hashmap of k-mers in the text.
    type Key = u64;
    let mut text_kmers = HashMap::<Key, SmallVec<[I; 4]>>::default();
    {
        let packed_text = PackedSeqVec::from_ascii(text);

        for i in (0..=n - k).step_by(k as _) {
            let kmer = packed_text.slice(i as _..(i + k) as _).to_word() as Key;
            text_kmers.entry(kmer).or_default().push(i);
        }
    }

    t.done("Indexing text");

    // 2. Set up helper functions.

    let potential_h = |Pos(i, _j)| -> I {
        // Number of following seeds.
        n / k - i / k
    };

    let transform = |p @ Pos(i, j)| {
        let p = potential_h(p);
        TPos(i - j - p, j - i - p)
    };

    let transform_back = |TPos(x, y)| -> Pos {
        if x == I::MAX && y == I::MAX {
            return Pos(x, y);
        }
        // p = n/k - i/k
        // i = k*(n/k-p)
        let p = -(x + y) / 2;
        let i = k * (n / k - p);
        let diff = (x - y) / 2;
        let j = i - diff;
        Pos(i, j)
    };

    // 3. Loop over patterns.
    for pat in patterns {
        let m = pat.len() as I;

        // 4. Find k-mer matches.
        // These are already transformed.
        let mut t_matches = vec![];
        {
            let packed_pat = PackedSeqVec::from_ascii(pat);
            for j in 0..=m - k {
                let kmer = packed_pat.slice(j as _..(j + k) as _).to_word() as Key;
                if let Some(is) = text_kmers.get(&kmer) {
                    t_matches.extend(is.iter().map(|&i| transform(Pos(i, j))));
                }
            }
        }
        t.done("Finding matches");
        s.add("Matches", t_matches.len());

        // 5. Sort matches
        // First left-to-right, then bottom-to-top.
        t_matches.sort_unstable_by_key(|&TPos(x, y)| (x, -y));
        t.done("Sorting matches");

        // 6. Do the chaining, via the classic LCP algorithm.
        let max_level = (m / k) as usize + 10;
        let mut front = vec![I::MIN + 1; max_level];
        let mut contours = vec![vec![]; max_level];

        front[0] = I::MAX;
        contours[0].push((TPos(I::MAX, I::MAX), true));

        for tm in t_matches.iter().rev() {
            // TODO: optimize joining to last or start.
            let y = tm.1 + 1;
            let layer = front.binary_search_by_key(&-y, |y| -y);
            let layer = layer.unwrap_or_else(|layer| layer - 1) + 1;
            assert!(layer > 0);

            contours[layer].push((*tm, true));
            front[layer] = max(front[layer], tm.1);
            // Set matches in layer-1 to non-dominant.

            let mut cnt = 0;
            for (tm2, dominant) in contours[layer - 1].iter_mut().rev() {
                if tm.0 <= tm2.0 && tm.1 <= tm2.1 {
                    *dominant = false;
                    cnt += 1;
                } else {
                    break;
                }
            }
            assert!(cnt > 0);
        }
        t.done("Building contours");

        // 7. Find sufficiently good local minima leading to dominant matches.
        let min_chain_length = ((m / k) as f32 * MIN_MATCH_FRACTION) as usize;
        let mut starts = vec![];
        for layer in (min_chain_length..contours.len()).rev() {
            for (tm, dominant) in &contours[layer] {
                if !dominant {
                    continue;
                }
                let m = transform_back(*tm);
                starts.push((m, layer));
            }
        }
        t.skip();
        s.add("Starts", starts.len());

        // for layer in 0..contours.len() {
        //     let ms = &contours[layer];
        //     eprintln!("Layer {layer} of len {}: {ms:?}", ms.len());
        //     if ms.is_empty() {
        //         break;
        //     }
        // }
    }
}

struct Timer {
    t0: Instant,
    t: Instant,
    keys: Vec<&'static str>,
    acc: HashMap<&'static str, std::time::Duration>,
}

impl Timer {
    fn new() -> Self {
        env_logger::init();
        Self {
            t0: Instant::now(),
            t: Instant::now(),
            keys: vec![],
            acc: HashMap::new(),
        }
    }

    fn done(&mut self, msg: &'static str) {
        let t = Instant::now();
        let elapsed = t - self.t;
        self.t = t;
        info!("{msg:>20}: {elapsed:>9.3?}");
        *self.acc.entry(msg).or_insert_with(|| {
            self.keys.push(msg);
            Duration::default()
        }) += elapsed;
    }

    fn skip(&mut self) {
        self.t = Instant::now();
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let total = Instant::now() - self.t0;
        info!("-------------------------------");
        info!("{:>20}  {total:>9.3?}", "TOTAL TIMES");
        for msg in &self.keys {
            let elapsed = self.acc[msg];
            info!("{msg:>20}: {elapsed:>9.3?}");
        }
    }
}

struct Stats {
    keys: Vec<&'static str>,
    acc: HashMap<&'static str, usize>,
}

impl Stats {
    fn new() -> Self {
        Self {
            keys: vec![],
            acc: HashMap::new(),
        }
    }

    fn add(&mut self, msg: &'static str, cnt: usize) {
        *self.acc.entry(msg).or_insert_with(|| {
            self.keys.push(msg);
            0
        }) += cnt;
        info!("{msg:>20}: {cnt:>9}");
    }
}

impl Drop for Stats {
    fn drop(&mut self) {
        info!("-------------------------------");
        info!("{:>20}", "TOTAL STATS");
        for msg in &self.keys {
            let cnt = self.acc[msg];
            info!("{msg:>20}: {cnt:>9}");
        }
    }
}
