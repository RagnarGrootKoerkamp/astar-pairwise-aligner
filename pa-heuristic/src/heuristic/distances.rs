use bio::alphabets::{Alphabet, RankTransform};

/// An O(1) evaluation heuristic that can be used to lower bound the distance between any two positions.
/// Used to get the distance between matches, instead of only distance to the end.
use crate::prelude::*;

use super::*;

// TODO: Can we get away with only one of these two traits?
pub trait Distance: Heuristic + Default {
    // TODO: Provide default implementations for these.
    type DistanceInstance<'a>: DistanceInstance<'a>;
    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>) -> Self::DistanceInstance<'a>;
}

pub trait DistanceInstance<'a>: HeuristicInstance<'a> {
    fn distance(&self, from: Pos, to: Pos) -> Cost;
}

// # NONE HEURISTIC
// The difference between None and Zero is that None is special cased is NW and
// DT implementations to fall back to simpler methods, while Zero is considered
// like any other heuristic.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoCost;
impl Heuristic for NoCost {
    type Instance<'a> = NoCostI;
    const IS_DEFAULT: bool = true;

    fn name(&self) -> String {
        "Zero".into()
    }

    fn build<'a>(&self, _a: Seq<'a>, _b: Seq<'a>) -> Self::Instance<'a> {
        NoCostI
    }
}
impl Distance for NoCost {
    type DistanceInstance<'a> = NoCostI;

    fn build<'a>(&self, _a: Seq<'a>, _b: Seq<'a>) -> Self::DistanceInstance<'a> {
        NoCostI
    }
}

pub struct NoCostI;
impl HeuristicInstance<'_> for NoCostI {
    fn h(&self, _pos: Pos) -> Cost {
        0
    }
}
impl DistanceInstance<'_> for NoCostI {
    fn distance(&self, _from: Pos, _to: Pos) -> Cost {
        0
    }
}

// # Zero HEURISTIC
#[derive(Debug, Clone, Copy, Default)]
pub struct ZeroCost;
impl Heuristic for ZeroCost {
    type Instance<'a> = ZeroCostI;

    fn name(&self) -> String {
        "None".into()
    }

    fn build<'a>(&self, _a: Seq<'a>, _b: Seq<'a>) -> Self::Instance<'a> {
        ZeroCostI
    }
}
impl Distance for ZeroCost {
    type DistanceInstance<'a> = ZeroCostI;

    fn build<'a>(&self, _a: Seq<'a>, _b: Seq<'a>) -> Self::DistanceInstance<'a> {
        ZeroCostI
    }
}

pub struct ZeroCostI;
impl HeuristicInstance<'_> for ZeroCostI {
    fn h(&self, _pos: Pos) -> Cost {
        0
    }
}
impl DistanceInstance<'_> for ZeroCostI {
    fn distance(&self, _from: Pos, _to: Pos) -> Cost {
        0
    }
}

// # MAX HEURISTIC
#[derive(Debug, Clone, Copy, Default)]
pub struct MaxCost;
impl Heuristic for MaxCost {
    type Instance<'a> = MaxCostI;
    fn name(&self) -> String {
        "Max".into()
    }

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>) -> Self::Instance<'a> {
        MaxCostI {
            target: Pos::target(a, b),
        }
    }
}
impl Distance for MaxCost {
    type DistanceInstance<'a> = MaxCostI;

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>) -> Self::DistanceInstance<'a> {
        <MaxCost as Heuristic>::build(self, a, b)
    }
}
pub struct MaxCostI {
    target: Pos,
}

impl HeuristicInstance<'_> for MaxCostI {
    fn h(&self, from: Pos) -> Cost {
        self.distance(from, self.target)
    }
}
impl DistanceInstance<'_> for MaxCostI {
    fn distance(&self, from: Pos, to: Pos) -> Cost {
        max(to.0 - from.0, to.1 - from.1) as Cost
    }
}

// # GAP HEURISTIC
#[derive(Debug, Clone, Copy, Default)]
pub struct GapCost;
impl Heuristic for GapCost {
    type Instance<'a> = GapCostI;
    fn name(&self) -> String {
        "Gap".into()
    }

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>) -> Self::Instance<'a> {
        GapCostI {
            target: Pos::target(a, b),
        }
    }
}
impl Distance for GapCost {
    type DistanceInstance<'a> = GapCostI;

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>) -> Self::DistanceInstance<'a> {
        <GapCost as Heuristic>::build(self, a, b)
    }
}
pub struct GapCostI {
    target: Pos,
}

fn abs_diff(i: I, j: I) -> I {
    (i as isize - j as isize).abs() as I
}

impl HeuristicInstance<'_> for GapCostI {
    fn h(&self, from: Pos) -> Cost {
        self.distance(from, self.target)
    }
}
impl DistanceInstance<'_> for GapCostI {
    fn distance(&self, from: Pos, to: Pos) -> Cost {
        abs_diff(to.0 - from.0, to.1 - from.1) as Cost
    }
}

// # COUNT HEURISTIC
// TODO: Make the 4 here variable.
type Counts = Vec<[usize; 4]>;
fn char_counts(a: Seq) -> Counts {
    let transform = RankTransform::new(&Alphabet::new(b"ACGT"));
    let mut counts = vec![[0; 4]];
    for idx in transform.qgrams(1, a) {
        counts.push(*counts.last().unwrap());
        counts.last_mut().unwrap()[idx] += 1;
    }
    counts
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CountCost;
impl Heuristic for CountCost {
    type Instance<'a> = CountCostI;
    fn name(&self) -> String {
        "Count".into()
    }

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>) -> Self::Instance<'a> {
        CountCostI {
            a_cnts: char_counts(a),
            b_cnts: char_counts(b),
            target: Pos::target(a, b),
        }
    }
}
impl Distance for CountCost {
    type DistanceInstance<'a> = CountCostI;

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>) -> Self::DistanceInstance<'a> {
        <CountCost as Heuristic>::build(self, a, b)
    }
}
pub struct CountCostI {
    a_cnts: Counts,
    b_cnts: Counts,
    target: Pos,
}

impl HeuristicInstance<'_> for CountCostI {
    fn h(&self, pos: Pos) -> Cost {
        self.distance(pos, self.target)
    }
}

impl DistanceInstance<'_> for CountCostI {
    fn distance(&self, from: Pos, to: Pos) -> Cost {
        let mut pos = 0;
        let mut neg = 0;

        // TODO: Find
        for (afrom, ato, bfrom, bto) in itertools::izip!(
            &self.a_cnts[from.0 as usize],
            &self.a_cnts[to.0 as usize],
            &self.b_cnts[from.1 as usize],
            &self.b_cnts[to.1 as usize],
        ) {
            let delta = (ato - afrom) as isize - (bto - bfrom) as isize;
            if delta > 0 {
                pos += delta;
            } else {
                neg -= delta;
            }
        }

        max(pos, neg) as Cost
    }
}

// # BICOUNT HEURISTIC
// Index i gives the bi-mer counts on a[0..i) (The half open interval.)
// NOTE: This is probably broken currently, since the triangle inequality
//   dist(A,C) <= dist(A, B) + dist(B, C)
// does not always hold, while that is assumed by the current implementation of SeedHeuristic.
// Maybe this can be fixed by returning floating point distances.
// TODO: Make the 4^2 here variable.
type BiCounts = Vec<[usize; 16]>;
fn char_bicounts(a: Seq) -> BiCounts {
    let transform = RankTransform::new(&Alphabet::new(b"ACGT"));
    let mut counts = vec![[0; 16]; 2];
    for idx in transform.qgrams(2, a) {
        counts.push(*counts.last().unwrap());
        counts.last_mut().unwrap()[idx] += 1;
    }
    counts.push(*counts.last().unwrap());
    counts
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BiCountCost;
impl Heuristic for BiCountCost {
    type Instance<'a> = BiCountCostI;
    fn name(&self) -> String {
        "BiCount".into()
    }

    fn build(&self, a: Seq, b: Seq) -> Self::Instance<'_> {
        BiCountCostI {
            cnt: Distance::build(&CountCost, a, b),
            a_cnts: char_bicounts(a),
            b_cnts: char_bicounts(b),
            target: Pos::target(a, b),
        }
    }
}
impl Distance for BiCountCost {
    type DistanceInstance<'a> = BiCountCostI;

    fn build(&self, a: Seq, b: Seq) -> Self::DistanceInstance<'_> {
        <BiCountCost as Heuristic>::build(self, a, b)
    }
}
pub struct BiCountCostI {
    cnt: CountCostI,
    a_cnts: BiCounts,
    b_cnts: BiCounts,
    target: Pos,
}

impl<'a> HeuristicInstance<'a> for BiCountCostI {
    fn h(&self, pos: Pos) -> Cost {
        self.distance(pos, self.target)
    }
}

impl<'a> DistanceInstance<'a> for BiCountCostI {
    fn distance(&self, from: Pos, to: Pos) -> Cost {
        let mut pos = 0;
        let mut neg = 0;

        // TODO: It should be possible to do some clever things here and use the
        // actual types of bimers to get a better lower bound.
        for (afrom, ato, bfrom, bto) in itertools::izip!(
            &self.a_cnts[min(from.0 + 1, to.0) as usize],
            &self.a_cnts[to.0 as usize],
            &self.b_cnts[min(from.1 + 1, to.1) as usize],
            &self.b_cnts[to.1 as usize],
        ) {
            let delta = (ato - afrom) as isize - (bto - bfrom) as isize;
            if delta > 0 {
                pos += delta;
            } else {
                neg -= delta;
            }
        }

        max(
            self.cnt.distance(from, to),
            // TODO: Why does rounding up give an error here?
            ((max(pos, neg) + 1) / 2) as Cost,
        )
    }
}

// # AFFINE GAP HEURISTIC
// NOTE: This currently assumes (x=1, o=1, e=1) and seedcost r=1.
#[derive(Debug, Clone, Copy, Default)]
pub struct AffineGapCost {
    pub k: I,
}
impl Heuristic for AffineGapCost {
    type Instance<'a> = AffineGapCostI;
    fn name(&self) -> String {
        "AffineGap".into()
    }

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>) -> Self::Instance<'a> {
        AffineGapCostI {
            k: self.k,
            target: Pos::target(a, b),
        }
    }
}
impl Distance for AffineGapCost {
    type DistanceInstance<'a> = AffineGapCostI;

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>) -> Self::DistanceInstance<'a> {
        <AffineGapCost as Heuristic>::build(self, a, b)
    }
}
pub struct AffineGapCostI {
    k: I,
    target: Pos,
}

impl HeuristicInstance<'_> for AffineGapCostI {
    fn h(&self, from: Pos) -> Cost {
        self.distance(from, self.target)
    }
}
impl DistanceInstance<'_> for AffineGapCostI {
    fn distance(&self, from: Pos, to: Pos) -> Cost {
        let e = (to.1 - to.0) - (from.1 - from.0);
        let s = to.0.div_floor(self.k) - from.0.div_ceil(self.k);
        //return max(e.abs(), s);
        // If on same diagonal
        match e {
            // Diagonal
            e if e == 0 => s,
            // Vertical
            e if e > 0 => s + e,
            // Horizontal
            // TODO: Make this more strict for large gaps
            e if e < 0 => s + e.abs(),
            // FIXME: Make this consistent
            //e if e < 0 => s + e.abs(),
            _ => unreachable!(),
        }
    }
}
