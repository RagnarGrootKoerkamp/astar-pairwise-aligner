/// An O(1) evaluation heuristic that can be used to lower bound the distance between any two positions.
/// Used to get the distance between matches, instead of only distance to the end.
use crate::prelude::*;

// TODO: Can we get away with only one of these two traits?
pub trait DistanceHeuristic: Heuristic
//where
//for<'a> Self::Instance<'a>: DistanceHeuristicInstance<'a>,
{
    // TODO: Provide default implementations for these.
    type DistanceInstance<'a>: DistanceHeuristicInstance<'a>;
    fn build<'a>(
        &self,
        a: &'a Sequence,
        b: &'a Sequence,
        alphabet: &Alphabet,
    ) -> Self::DistanceInstance<'a>;
}

pub trait DistanceHeuristicInstance<'a>: HeuristicInstance<'a> {
    fn distance(&self, from: Self::Pos, to: Self::Pos) -> usize;
}

// # ZERO HEURISTIC
#[derive(Debug, Clone, Copy)]
pub struct ZeroHeuristic;
impl Heuristic for ZeroHeuristic {
    type Instance<'a> = ZeroHeuristicI;

    fn name(&self) -> String {
        "Zero".into()
    }

    fn build<'a>(
        &self,
        _a: &'a Sequence,
        _b: &'a Sequence,
        _alphabet: &Alphabet,
    ) -> Self::Instance<'a> {
        ZeroHeuristicI
    }
}
impl DistanceHeuristic for ZeroHeuristic {
    type DistanceInstance<'a> = ZeroHeuristicI;

    fn build<'a>(
        &self,
        _a: &'a Sequence,
        _b: &'a Sequence,
        _alphabet: &Alphabet,
    ) -> Self::DistanceInstance<'a> {
        ZeroHeuristicI
    }
}

pub struct ZeroHeuristicI;
impl HeuristicInstance<'_> for ZeroHeuristicI {
    fn h(&self, _: NodeH<Self>) -> usize {
        0
    }
}
impl DistanceHeuristicInstance<'_> for ZeroHeuristicI {
    fn distance(&self, _from: Pos, _to: Pos) -> usize {
        0
    }
}

// # GAP HEURISTIC
#[derive(Debug, Clone, Copy)]
pub struct GapHeuristic;
impl Heuristic for GapHeuristic {
    type Instance<'a> = GapHeuristicI;
    fn name(&self) -> String {
        "Gap".into()
    }

    fn build<'a>(
        &self,
        a: &'a Sequence,
        b: &'a Sequence,
        _alphabet: &Alphabet,
    ) -> Self::Instance<'a> {
        GapHeuristicI {
            target: Pos(a.len(), b.len()),
        }
    }
}
impl DistanceHeuristic for GapHeuristic {
    type DistanceInstance<'a> = GapHeuristicI;

    fn build<'a>(
        &self,
        a: &'a Sequence,
        b: &'a Sequence,
        alphabet: &Alphabet,
    ) -> Self::DistanceInstance<'a> {
        <GapHeuristic as Heuristic>::build(self, a, b, alphabet)
    }
}
pub struct GapHeuristicI {
    target: Pos,
}

impl HeuristicInstance<'_> for GapHeuristicI {
    fn h(&self, Node(Pos(i, j), _): NodeH<Self>) -> usize {
        abs_diff(self.target.0 - i, self.target.1 - j)
    }
}
impl DistanceHeuristicInstance<'_> for GapHeuristicI {
    fn distance(&self, from: Pos, to: Pos) -> usize {
        abs_diff(to.0 - from.0, to.1 - from.1)
    }
}

// # COUNT HEURISTIC
// TODO: Make the 4 here variable.
type Counts = Vec<[usize; 4]>;
fn char_counts(a: &Sequence, alphabet: &Alphabet) -> Counts {
    let transform = RankTransform::new(alphabet);
    let mut counts = vec![[0; 4]];
    for idx in transform.qgrams(1, a) {
        counts.push(*counts.last().unwrap());
        counts.last_mut().unwrap()[idx] += 1;
    }
    counts
}

#[derive(Debug, Clone, Copy)]
pub struct CountHeuristic;
impl Heuristic for CountHeuristic {
    type Instance<'a> = CountHeuristicI;
    fn name(&self) -> String {
        "Count".into()
    }

    fn build<'a>(
        &self,
        a: &'a Sequence,
        b: &'a Sequence,
        alphabet: &Alphabet,
    ) -> Self::Instance<'a> {
        CountHeuristicI {
            a_cnts: char_counts(a, alphabet),
            b_cnts: char_counts(b, alphabet),
            target: Pos(a.len(), b.len()),
        }
    }
}
impl DistanceHeuristic for CountHeuristic {
    type DistanceInstance<'a> = CountHeuristicI;

    fn build<'a>(
        &self,
        a: &'a Sequence,
        b: &'a Sequence,
        alphabet: &Alphabet,
    ) -> Self::DistanceInstance<'a> {
        <CountHeuristic as Heuristic>::build(self, a, b, alphabet)
    }
}
pub struct CountHeuristicI {
    a_cnts: Counts,
    b_cnts: Counts,
    target: Pos,
}

impl HeuristicInstance<'_> for CountHeuristicI {
    fn h(&self, Node(pos, _): NodeH<Self>) -> usize {
        self.distance(pos, self.target)
    }
}

impl DistanceHeuristicInstance<'_> for CountHeuristicI {
    fn distance(&self, from: Pos, to: Pos) -> usize {
        let mut pos = 0;
        let mut neg = 0;

        // TODO: Find
        for (afrom, ato, bfrom, bto) in itertools::izip!(
            &self.a_cnts[from.0],
            &self.a_cnts[to.0],
            &self.b_cnts[from.1],
            &self.b_cnts[to.1],
        ) {
            let delta = (ato - afrom) as isize - (bto - bfrom) as isize;
            if delta > 0 {
                pos += delta;
            } else {
                neg -= delta;
            }
        }

        max(pos, neg) as usize
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
fn char_bicounts(a: &Sequence, alphabet: &Alphabet) -> BiCounts {
    let transform = RankTransform::new(alphabet);
    let mut counts = vec![[0; 16]; 2];
    for idx in transform.qgrams(2, a) {
        counts.push(*counts.last().unwrap());
        counts.last_mut().unwrap()[idx] += 1;
    }
    counts.push(*counts.last().unwrap());
    counts
}

#[derive(Debug, Clone, Copy)]
pub struct BiCountHeuristic;
impl Heuristic for BiCountHeuristic {
    type Instance<'a> = BiCountHeuristicI;
    fn name(&self) -> String {
        "BiCount".into()
    }

    fn build(&self, a: &Sequence, b: &Sequence, alphabet: &Alphabet) -> Self::Instance<'_> {
        BiCountHeuristicI {
            cnt: DistanceHeuristic::build(&CountHeuristic, a, b, alphabet),
            a_cnts: char_bicounts(a, alphabet),
            b_cnts: char_bicounts(b, alphabet),
            target: Pos(a.len(), b.len()),
        }
    }
}
impl DistanceHeuristic for BiCountHeuristic {
    type DistanceInstance<'a> = BiCountHeuristicI;

    fn build(&self, a: &Sequence, b: &Sequence, alphabet: &Alphabet) -> Self::DistanceInstance<'_> {
        <BiCountHeuristic as Heuristic>::build(self, a, b, alphabet)
    }
}
pub struct BiCountHeuristicI {
    cnt: CountHeuristicI,
    a_cnts: BiCounts,
    b_cnts: BiCounts,
    target: Pos,
}

impl<'a> HeuristicInstance<'a> for BiCountHeuristicI {
    fn h(&self, Node(pos, _): NodeH<Self>) -> usize {
        self.distance(pos, self.target)
    }
}

impl<'a> DistanceHeuristicInstance<'a> for BiCountHeuristicI {
    fn distance(&self, from: Pos, to: Pos) -> usize {
        let mut pos = 0;
        let mut neg = 0;

        // TODO: It should be possible to do some clever things here and use the
        // actual types of bimers to get a better lower bound.
        for (afrom, ato, bfrom, bto) in itertools::izip!(
            &self.a_cnts[min(from.0 + 1, to.0)],
            &self.a_cnts[to.0],
            &self.b_cnts[min(from.1 + 1, to.1)],
            &self.b_cnts[to.1],
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
            ((max(pos, neg) + 1) / 2) as usize,
        )
    }
}
