//! Given a pattern, find all occurrences of the pattern in the text.
//! Basically, compute the full semi-global alignment table and return the bottom row of costs.
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

use std::{
    cmp::max,
    collections::HashMap,
    fs::File,
    hint::black_box,
    io::BufReader,
    path::PathBuf,
    time::{Duration, Instant},
};

use astarpa2::{AstarPa2, AstarPa2Params, AstarPa2Stats, Domain};
use bio::io::fasta;
use clap::{value_parser, Parser};
use fxhash::FxHashMap;
use itertools::Itertools;
use log::{info, trace, warn};
use pa_heuristic::{HeuristicInstance, HeuristicStats, LengthConfig, MatchConfig, Pruning, GCSH};
use pa_types::{Cost, Pos, I};
use pa_vis::NoVis;
use packed_seq::{PackedSeqVec, Seq, SeqVec};
use rdst::RadixKey;

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

    #[clap(long)]
    pub v2: bool,

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
            map(text.seq(), &patterns, args.k as I, args.v2, args.lp);
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

fn map(text: &[u8], patterns: &[&[u8]], k: I, v2: bool, lp: usize) {
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

    let mut stats = AstarPa2Stats::default();
    let mut h_stats = HeuristicStats::default();

    // 3. Loop over patterns.
    for pat in patterns {
        s.add("pattern", 1);
        // FIXME Reduce to multiple of 64 for simplicity for now.
        let pat = &pat[..(pat.len() / 64) * 64];
        warn!("LEN {}", pat.len());

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
        s.avg("Matches", t_matches.len());

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

        let mut last_layer: usize = 0;
        let mut hits = 0;
        let mut lhits = 0;
        let mut bss = 0;
        for tm in t_matches.iter().rev() {
            // TODO: optimize joining to last or start.
            let target_y = tm.1 + 1;

            // Try at top.
            let cnt = front[0..8].iter().filter(|&&y| y >= target_y).count();

            // HOT
            let layer = if cnt < 8 {
                hits += 1;
                cnt
            } else {
                // Try at last layer.
                let o = last_layer.saturating_sub(5);
                let cnt = front[o..o + 8].iter().filter(|&&y| y >= target_y).count();
                if 0 < cnt && cnt < 8 {
                    lhits += 1;
                    o + cnt
                } else {
                    bss += 1;
                    // Binary search.
                    let layer = front.binary_search_by_key(&-target_y, |y| -y);
                    let new_layer = layer.unwrap_or_else(|layer| layer - 1) + 1;
                    // eprintln!("{last_layer} -> {new_layer}");
                    new_layer
                }
            };
            assert!(layer > 0);

            if layer >= 7 {
                last_layer = layer;
            }

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
        s.avg("hits", hits);
        s.avg("lhits", lhits);
        s.avg("bss", bss);

        let max_layer = contours.iter().rposition(|x| !x.is_empty()).unwrap();
        s.avg("max_layer", max_layer);

        t.done("Building contours");

        // 7. Find sufficiently good local minima leading to dominant matches.
        let min_chain_length = ((m / k) as f32 * MIN_MATCH_FRACTION) as usize;
        s.once("min_chain", min_chain_length);

        let mut starts: Vec<(TPos, Pos, usize)> = vec![];
        for layer in (min_chain_length..contours.len()).rev() {
            'tm: for &(tm, dominant) in &contours[layer] {
                if !dominant {
                    continue;
                }
                let m = transform_back(tm);
                let si = m.0 - m.1;
                for (tm2, m2, _) in &starts {
                    if tm2.0 <= tm.0 && tm2.1 <= tm.1 {
                        continue 'tm;
                    }
                    if m2.0.abs_diff(si) < 100 {
                        continue 'tm;
                    }
                }
                starts.push((tm, Pos(si, 0), layer));
                trace!("start layer {layer:>4} at {m} original {tm:?} start idx {si}")
            }
        }
        // t.done("Starts");
        t.skip();
        s.avg("Starts", starts.len());

        // for layer in 0..contours.len() {
        //     let ms = &contours[layer];
        //     if ms.len() > 1 {
        //         eprintln!("Layer {layer} of len {}", ms.len());
        //     }
        //     if ms.is_empty() {
        //         break;
        //     }
        // }

        let mut best_cost = Cost::MAX;
        let mut next_best_cost = Cost::MAX;
        let mut best_result = None;
        for (_t_start, start, _layer) in starts {
            let sub_ref =
                &text[(start.0 as usize).saturating_sub(100)..start.0 as usize + pat.len() + 100];
            let mut result = None;
            let cost = if !v2 {
                // N*M BITPACKING

                // For now, simply fill the square part.
                result = Some(pa_bitpacking::search::search(pat, sub_ref, 1.0));
                *result.as_ref().unwrap().out.iter().min().unwrap()
            } else {
                // A*PA2 semi-global
                // simple
                // let mut params = AstarPa2Params::simple();
                // params.heuristic.heuristic = pa_heuristic::HeuristicType::SemiGlobalGap;

                // full
                let mut params = AstarPa2Params::full();
                params.heuristic.k = k;
                params.heuristic.p = lp;

                params.front.incremental_doubling = false;
                params.block_width = 128;

                let match_config = MatchConfig {
                    length: LengthConfig::Fixed(params.heuristic.k),
                    r: params.heuristic.r,
                    local_pruning: params.heuristic.p,
                };
                let pruning = Pruning {
                    enabled: params.heuristic.prune,
                    skip_prune: params.heuristic.skip_prune,
                };

                let aligner = AstarPa2 {
                    domain: Domain::Astar(GCSH::new(match_config, pruning)),
                    doubling: params.doubling,
                    block_width: params.block_width,
                    v: NoVis,
                    block: params.front,
                    trace: true,
                    sparse_h: params.sparse_h,
                    prune: params.prune,
                };

                if false {
                    aligner.align(sub_ref, pat).0
                } else {
                    let mut nw = aligner.build(sub_ref, pat);
                    t.done("build h");
                    let mut blocks = aligner.block.new(true, sub_ref, pat);
                    let (cost, cigar) = black_box(
                        nw.align_for_bounded_dist(Some(2300), true, Some(&mut blocks))
                            .unwrap(),
                    );
                    t.done("Align");
                    black_box(cigar);
                    nw.stats.block_stats += blocks.stats;
                    stats += nw.stats;
                    h_stats += nw.domain.h_mut().unwrap().stats();
                    cost
                }
            };
            trace!("cost {cost}");
            if cost < best_cost {
                next_best_cost = best_cost;
                best_cost = cost;
                best_result = result;
            } else if cost < next_best_cost {
                next_best_cost = cost;
            }
            break;
        }
        assert!(best_cost < Cost::MAX);
        s.avg("Best cost", best_cost as usize);
        if next_best_cost < Cost::MAX {
            s.avg("Next best cost", next_best_cost as usize);
        }
        t.done("Align");

        drop(best_result);
        // if let Some(r) = best_result {
        //     let idx = r.out.iter().position_min().unwrap();
        //     let (_cigar, path) = r.trace(idx);
        //     trace!(
        //         "Path from {} to {}",
        //         path.first().unwrap(),
        //         path.last().unwrap()
        //     );
        // }
        // t.done("Trace");
    }

    info!("A* STATS\n{stats:#?}");
    info!("H  STATS\n{h_stats:#?}");
}

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
    // RadixSort::radix_sort_unstable(&mut text_kmers);
    // Single threaded.
    radsort::sort_by_key(&mut text_kmers, |T(kmer, _)| *kmer);

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
        env_logger::Builder::from_default_env()
            .format_timestamp(None)
            // .format_timestamp_millis()
            .init();

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
        info!("{msg:<30}: {elapsed:>9.3?}");
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
        info!("-----------------------------------------");
        info!("{:<30}  {total:>9.3?}", "TOTAL TIMES");
        for msg in &self.keys {
            let elapsed = self.acc[msg];
            info!("{msg:<30}: {elapsed:>9.3?}");
        }
    }
}

struct Stats {
    keys: Vec<&'static str>,
    acc: HashMap<&'static str, usize>,
    cnts: HashMap<&'static str, usize>,
}

impl Stats {
    fn new() -> Self {
        Self {
            keys: vec![],
            acc: HashMap::new(),
            cnts: HashMap::new(),
        }
    }

    fn once(&mut self, msg: &'static str, cnt: usize) {
        self.acc.entry(msg).or_insert_with(|| {
            self.keys.push(msg);
            cnt
        });
        info!("{msg:<30}: {cnt:>9}");
    }

    fn add(&mut self, msg: &'static str, cnt: usize) {
        let sum = self.acc.entry(msg).or_insert_with(|| {
            self.keys.push(msg);
            0
        });
        *sum += cnt;
        info!("{msg:<30}: {cnt:>9} ({sum:>9})");
    }

    fn avg(&mut self, msg: &'static str, cnt: usize) {
        *self.acc.entry(msg).or_insert_with(|| {
            self.keys.push(msg);
            0
        }) += cnt;
        *self.cnts.entry(msg).or_default() += 1;
        info!("{msg:<30}: {cnt:>9}");
    }
}

impl Drop for Stats {
    fn drop(&mut self) {
        info!("-----------------------------------------");
        info!("{:<30}", "TOTAL STATS");
        for msg in &self.keys {
            let val = self.acc[msg];
            if let Some(cnt) = self.cnts.get(msg) {
                let avg = val as f32 / *cnt as f32;
                info!("{msg:<30}: {avg:>11.1} avg");
            } else {
                info!("{msg:<30}: {val:>9}");
            }
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
