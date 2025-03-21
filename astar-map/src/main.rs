//! Given a pattern, find all occurrences of the pattern in the text.
//! Basically, compute the full semi-global alignment table and return the bottom row of costs.
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

use std::{
    cmp::max,
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::PathBuf,
    time::{Duration, Instant},
};

use bio::io::fasta;
use clap::{value_parser, Parser};
use fxhash::FxHashMap;
use itertools::Itertools;
use log::info;
use pa_types::{Pos, I};
use packed_seq::{PackedSeqVec, Seq, SeqVec};
use rdst::{RadixKey, RadixSort};

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

    // let mut all_stats = AstarStats::default();

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
                todo!();
                // let k = args.k as _;
                // let mut match_config = MatchConfig::exact(k);
                // match_config.local_pruning = args.lp;

                // let pruning1 = Pruning::new(Prune::None);

                // let h = CSH {
                //     match_config,
                //     pruning: pruning1,
                //     use_gap_cost: true,
                //     c: PhantomData::<HintContours<SortedContour>>,
                // };
                // let ((cost, _cigar), stats) = AstarPa {
                //     dt: true,
                //     h,
                //     v: NoVis,
                // }
                // .align(text.seq(), seq);
                // eprintln!("Cost {cost}");
                // eprintln!("stats {stats:?}");
                // all_stats += stats;
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
    // if !args.map {
    //     eprintln!("ALL STATS\n{all_stats:?}");
    // }
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

type Key = u64;

fn map(text: &[u8], patterns: &[&[u8]], k: I) {
    let n = text.len() as I;

    let mut t = Timer::new();
    let mut s = Stats::new();

    // 1. Build hashmap of k-mers in the text.
    // In three steps:
    // 1a: collect all kmers into a vector.
    // 1b: sort the vector.
    // 1c: build a hashmap mapping kmers to slices.

    let (idx, pos) = index_text(text, k, &mut t);

    // 2. Set up helper functions.

    let divk = FM32::new(k as u32);
    let nk = n / k;

    let potential_h = |Pos(i, _j)| -> I {
        // Number of following seeds.
        nk - divk.fastdiv(i as u32) as i32
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
            t.done("Pack pattern");
            for j in 0..=m - k {
                let kmer = packed_pat.slice(j as _..(j + k) as _).to_word() as Key;
                if let Some(&(start, end)) = idx.get(&kmer) {
                    let is = &pos[start as usize..end as usize];
                    t_matches.extend(is.iter().map(
                        #[inline(always)]
                        |&i| transform(Pos(i, j)),
                    ));
                }
            }
        }
        t.done("Finding matches");
        s.add("Matches", t_matches.len());

        // 5. Sort matches
        // First left-to-right, then bottom-to-top.
        // single-threaded radsort.
        // TODO: Compress the range of y so that fewer rounds are needed for it.
        // It should be sufficient to sort by j only, which has smaller range.
        // HOT
        radsort::sort_by_key(&mut t_matches, |&TPos(x, y)| (x, -y));
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

#[inline(never)]
fn index_text(
    text: &[u8],
    k: i32,
    t: &mut Timer,
) -> (
    HashMap<u64, (u32, u32), std::hash::BuildHasherDefault<fxhash::FxHasher>>,
    Vec<i32>,
) {
    let n = text.len() as I;

    let mut text_kmers = vec![];
    text_kmers.reserve((n / k) as usize);
    let packed_text = PackedSeqVec::from_ascii(text);
    t.done("Pack text");

    for i in (0..=n - k).step_by(k as _) {
        let kmer = packed_text.slice(i as _..(i + k) as _).to_word() as Key;
        text_kmers.push(T(kmer, i));
    }
    t.done("Indexing text: collect");
    // Multithreaded building of the index.
    text_kmers.radix_sort_unstable();

    t.done("Indexing text: sort");

    let mut idx = FxHashMap::<Key, (u32, u32)>::default();
    idx.reserve((n / k) as usize);

    let mut start = 0;
    for (key, group) in text_kmers.iter().group_by(|T(kmer, _)| *kmer).into_iter() {
        let cnt = group.count() as u32;
        // HOT
        idx.insert(key, (start, start + cnt));
        start += cnt;
    }
    t.done("Indexing text: idx");
    let pos = text_kmers.into_iter().map(|T(_kmer, i)| i).collect_vec();
    t.done("Indexing text: shrink");
    (idx, pos)
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

#[derive(Copy, Clone)]
struct T(u64, i32);

impl RadixKey for T {
    const LEVELS: usize = 8;

    fn get_level(&self, level: usize) -> u8 {
        (self.0 >> (level * 8)) as u8
    }
}

/// FastMod32, using the low 32 bits of the hash.
/// Taken from https://github.com/lemire/fastmod/blob/master/include/fastmod.h
#[derive(Copy, Clone, Debug)]
struct FM32 {
    // d: u64,
    m: u64,
}
impl FM32 {
    fn new(d: u32) -> Self {
        Self {
            // d: d as u64,
            m: u64::MAX / d as u64 + 1,
        }
    }
    // fn fastmod(self, h: u32) -> usize {
    //     let lowbits = self.m.wrapping_mul(h as u64);
    //     ((lowbits as u128 * self.d as u128) >> 64) as usize
    // }
    fn fastdiv(self, h: u32) -> usize {
        ((self.m as u128 * h as u128) >> 64) as u32 as usize
    }
}
