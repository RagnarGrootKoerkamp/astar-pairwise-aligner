/// An O(1) evaluation heuristic that can be used to lower bound the distance between any two positions.
/// Used to get the distance between matches, instead of only distance to the end.
use crate::prelude::*;

// TODO: Can we get away with only one of these two traits?
pub trait Distance: Heuristic + Default {
    // TODO: Provide default implementations for these.
    type DistanceInstance<'a>: DistanceInstance<'a>;
    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>, alphabet: &Alphabet) -> Self::DistanceInstance<'a>;
}

pub trait DistanceInstance<'a>: HeuristicInstance<'a> {
    fn distance(&self, from: Pos, to: Pos) -> Cost;
}

// # ZERO HEURISTIC
#[derive(Debug, Clone, Copy, Default)]
pub struct ZeroCost;
impl Heuristic for ZeroCost {
    type Instance<'a> = ZeroCostI;
    const IS_DEFAULT: bool = true;

    fn name(&self) -> String {
        "Zero".into()
    }

    fn build<'a>(&self, _a: Seq<'a>, _b: Seq<'a>, _alphabet: &Alphabet) -> Self::Instance<'a> {
        ZeroCostI
    }
}
impl Distance for ZeroCost {
    type DistanceInstance<'a> = ZeroCostI;

    fn build<'a>(
        &self,
        _a: Seq<'a>,
        _b: Seq<'a>,
        _alphabet: &Alphabet,
    ) -> Self::DistanceInstance<'a> {
        ZeroCostI
    }
}

pub struct ZeroCostI;
impl HeuristicInstance<'_> for ZeroCostI {
    fn h(&self, pos: Pos) -> Cost {
        println!("ZeroCost in {pos}");
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

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>, _alphabet: &Alphabet) -> Self::Instance<'a> {
        MaxCostI {
            target: Pos::from_lengths(a, b),
        }
    }
}
impl Distance for MaxCost {
    type DistanceInstance<'a> = MaxCostI;

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>, alphabet: &Alphabet) -> Self::DistanceInstance<'a> {
        <MaxCost as Heuristic>::build(self, a, b, alphabet)
    }
}
pub struct MaxCostI {
    target: Pos,
}

impl HeuristicInstance<'_> for MaxCostI {
    fn h(&self, Pos(i, j): Pos) -> Cost {
        max(self.target.0 - i, self.target.1 - j) as Cost
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

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>, _alphabet: &Alphabet) -> Self::Instance<'a> {
        GapCostI {
            target: Pos::from_lengths(a, b),
        }
    }
}
impl Distance for GapCost {
    type DistanceInstance<'a> = GapCostI;

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>, alphabet: &Alphabet) -> Self::DistanceInstance<'a> {
        <GapCost as Heuristic>::build(self, a, b, alphabet)
    }
}
pub struct GapCostI {
    target: Pos,
}

pub fn abs_diff(i: I, j: I) -> I {
    (i as isize - j as isize).abs() as u32
}

impl HeuristicInstance<'_> for GapCostI {
    fn h(&self, Pos(i, j): Pos) -> Cost {
        abs_diff(self.target.0 - i, self.target.1 - j) as Cost
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
fn char_counts(a: Seq, alphabet: &Alphabet) -> Counts {
    let transform = RankTransform::new(alphabet);
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

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>, alphabet: &Alphabet) -> Self::Instance<'a> {
        CountCostI {
            a_cnts: char_counts(a, alphabet),
            b_cnts: char_counts(b, alphabet),
            target: Pos::from_lengths(a, b),
        }
    }
}
impl Distance for CountCost {
    type DistanceInstance<'a> = CountCostI;

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>, alphabet: &Alphabet) -> Self::DistanceInstance<'a> {
        <CountCost as Heuristic>::build(self, a, b, alphabet)
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
fn char_bicounts(a: Seq, alphabet: &Alphabet) -> BiCounts {
    let transform = RankTransform::new(alphabet);
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

    fn build(&self, a: Seq, b: Seq, alphabet: &Alphabet) -> Self::Instance<'_> {
        BiCountCostI {
            cnt: Distance::build(&CountCost, a, b, alphabet),
            a_cnts: char_bicounts(a, alphabet),
            b_cnts: char_bicounts(b, alphabet),
            target: Pos::from_lengths(a, b),
        }
    }
}
impl Distance for BiCountCost {
    type DistanceInstance<'a> = BiCountCostI;

    fn build(&self, a: Seq, b: Seq, alphabet: &Alphabet) -> Self::DistanceInstance<'_> {
        <BiCountCost as Heuristic>::build(self, a, b, alphabet)
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
