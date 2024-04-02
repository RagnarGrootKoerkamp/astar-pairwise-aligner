use bio::alphabets::{Alphabet, RankTransform};

/// An O(1) evaluation heuristic that can be used to lower bound the distance between any two positions.
/// Used to get the distance between matches, instead of only distance to the end.
use crate::prelude::*;

use super::*;

// TODO: Can we get away with only one of these two traits?
pub trait Distance: Heuristic {
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
        let d = (to.1 - to.0) - (from.1 - from.0);
        let p = to.0.div_floor(self.k) - from.0.div_ceil(self.k);
        //return max(e.abs(), s);
        // If on same diagonal
        match d {
            // Diagonal
            0 => p,
            // Vertical
            d if d > 0 => p + d,
            // Horizontal
            d if d < 0 => p + d.abs(),
            _ => unreachable!(),
        }
    }
}

// # AFFINE GAP SEED HEURISTIC
// Returns the distance between two states, taking into account both the gapcost
// and seedcost.
// NOTE: This currently assumes (x=1, o=1, e=1) and seedcost r=1.
#[derive(Debug, Clone, Copy)]
pub struct SimpleAffineCost {
    pub sub: I,
    pub open: I,
    pub extend: I,
}
#[derive(Debug, Clone, Copy)]
pub struct AffineGapSeedCost {
    pub k: I,
    pub r: I,
    pub c: SimpleAffineCost,
    pub formula: bool,
}
impl Heuristic for AffineGapSeedCost {
    type Instance<'a> = AffineGapSeedCostI;
    fn name(&self) -> String {
        "AffineGap".into()
    }

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>) -> Self::Instance<'a> {
        AffineGapSeedCostI {
            params: *self,
            target: Pos::target(a, b),
        }
    }
}
impl Distance for AffineGapSeedCost {
    type DistanceInstance<'a> = AffineGapSeedCostI;

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>) -> Self::DistanceInstance<'a> {
        <AffineGapSeedCost as Heuristic>::build(self, a, b)
    }
}
pub struct AffineGapSeedCostI {
    params: AffineGapSeedCost,
    target: Pos,
}

impl HeuristicInstance<'_> for AffineGapSeedCostI {
    fn h(&self, from: Pos) -> Cost {
        self.distance(from, self.target)
    }
}
impl DistanceInstance<'_> for AffineGapSeedCostI {
    fn distance(&self, from: Pos, to: Pos) -> Cost {
        let AffineGapSeedCost { k, r, c, formula } = self.params;
        let SimpleAffineCost { open, extend, .. } = c;

        // #diagonals to change
        let d = (to.1 - to.0) - (from.1 - from.0);
        // #seed crossed
        let p = to.0.div_floor(k) - from.0.div_ceil(k);
        let p = max(p, 0);
        assert!(p >= 0, "p: {}, from: {:?}, to: {:?}", p, from, to);

        if d == 0 {
            return p * r;
        }

        if p == 0 {
            return c.open + c.extend * d.abs();
        }

        if true {
            // import math

            // def cost_make_all_insertions_in_one_seed(p, d, seed_potential, indel_cost, gap_open_cost):
            //     # Make all insertions in one seed. There must be a seed to make insertions in though.
            //     # Does not depend on whether indels are horizontal or vertical as all indels are in one seed anyway.
            //     cost = (p - 1) * seed_potential + max((gap_open_cost if abs(d) else 0) + indel_cost * abs(d), seed_potential) if p else float("inf")
            //     return cost

            // def distribute_indels_evenly(p, d):
            //     # Distribute insertions evenly across seeds
            //     indels_per_seed = [0 for _ in range(p)]
            //     at_seed, indels_left = 0, abs(d)
            //     while p and indels_left:
            //         indels_per_seed[at_seed] += 1
            //         at_seed = (at_seed + 1) % p
            //         indels_left -= 1
            //     return indels_per_seed, indels_left

            // def chain_indels(p, d, seed_length):
            //     assert (
            //         0 < abs(d) <= p * seed_length
            //     ), f"Expected 0 < abs(d) <= p * seed_length, got abs(d)={abs(d)} and p * seed_length={p * seed_length}"

            //     # Chain all insertions together
            //     indels_per_seed = [0 for _ in range(p)]
            //     at_seed, indels_left = 0, abs(d)
            //     while at_seed < p and indels_left:
            //         indels_per_seed[at_seed] = min(seed_length, indels_left)
            //         indels_left -= indels_per_seed[at_seed]
            //         at_seed += 1

            //     # There should be no indels left since 0 < abs(d) <= p * seed_length
            //     assert indels_left == 0, f"Expected indels_left == 0, got {indels_left}"

            //     return indels_per_seed

            // def distribute_gaps(p, d, seed_length, indels_per_seed):
            //     seed_is_full = [indels >= seed_length for indels in indels_per_seed]
            //     n_full_seeds = sum(seed_is_full)
            //     # Only count full seeds as one and then non-full seeds with indels.
            //     n_gapped_seeds = sum([1 if not seed_is_full[i] and indels_per_seed[i] else 0 for i in range(p)]) + (1 if n_full_seeds else 0)
            //     n_gaps_needed = max(1 if d else 0, math.ceil(n_gapped_seeds / 2))

            //     gap_in_seeds = [0 for _ in range(p)]
            //     n_gaps_to_distribute = n_gaps_needed
            //     # We need to open a gap in the first seed if there are any gaps to distribute
            //     if p and n_gaps_to_distribute:
            //         gap_in_seeds[0] = 1
            //         n_gaps_to_distribute -= 1
            //         at_seed = 1
            //     # Chain as many full seeds together as possible
            //     while at_seed < p and n_gaps_to_distribute and seed_is_full[at_seed]:
            //         at_seed += 1

            //     # If there are even number of gapped seeds left then we group them together two by two.
            //     # If there are odd number of gapped seeds left then it's better to # have the left-most
            //     #   seed alone and group the rest two by two.
            //     if at_seed + 1 < p and n_gaps_to_distribute and n_gapped_seeds % 2 == 1:
            //         gap_in_seeds[at_seed] = 1
            //         n_gaps_to_distribute -= 1
            //     for i in range(at_seed + 1, p, 2):
            //         if n_gaps_to_distribute == 0:
            //             break
            //         gap_in_seeds[i] = 1
            //         n_gaps_to_distribute -= 1

            //     return gap_in_seeds

            // def cost_vertical_insertions(seed_potential, indel_cost, gap_open_cost, indels_per_seed, indels_left):
            //     # If the insertions are vertical, then we cannot chain insertions across seeds
            //     cost = (
            //         (gap_open_cost if indels_left else 0)
            //         + indels_left * indel_cost
            //         + sum([max(seed_potential, (gap_open_cost if indels else 0) + indels * indel_cost) for indels in indels_per_seed])
            //     )
            //     return cost

            // def cost_horizontal_insertions(p, d, seed_potential, indel_cost, gap_open_cost, seed_length, indels_per_seed, indels_left):
            //     # If the insertions are horizontal, then we might be able to chain insertions across seeds and open fewer gaps.
            //     # This kinda assumes that seed_length * p >= d, that is, that we can cross all diagonals with the seeds.
            //     gap_in_seeds = distribute_gaps(p, d, seed_length, indels_per_seed)
            //     cost = (
            //         (gap_open_cost if indels_left else 0)
            //         + indels_left * indel_cost
            //         + sum(
            //             [
            //                 max(seed_potential, gap_open_cost * gap_in_seed + indels * indel_cost)
            //                 for indels, gap_in_seed in zip(indels_per_seed, gap_in_seeds)
            //             ]
            //         )
            //     )
            //     return cost

            // def cost_chain_horizontal_insertions(p, d, seed_potential, indel_cost, gap_open_cost, seed_length):
            //     # Chain all insertions together
            //     indels_per_seed = chain_indels(p, d, seed_length)

            //     at_seed = p
            //     for i in range(p):
            //         if indels_per_seed[i] != seed_length:
            //             at_seed = i
            //             break

            //     # We need to open a gap in the first seed but in the rest of the seeds we chain the insertions.
            //     cost_b_chained = max(gap_open_cost + indel_cost * indels_per_seed[0], seed_potential)
            //     for i in range(1, p):
            //         cost_b_chained += max(indel_cost * indels_per_seed[i], seed_potential)

            //     # Check if the previous seed is fully saturated
            //     if indels_per_seed[at_seed - 1] != seed_length:
            //         at_seed -= 1
            //     # If only the first seed is saturated then we can move insertions to the next seed and still chain them
            //     if at_seed == 0:
            //         at_seed += 1

            //     for i in range(indels_per_seed[0]):
            //         # at_seed == 0 is covered by inserting all insertions in one seed
            //         # at_seed >= p means we can't move insertions to the next seed
            //         if at_seed == 0 or at_seed >= p:
            //             break
            //         indels_per_seed[0] -= 1
            //         indels_per_seed[at_seed] += 1
            //         # print(f"  new {indels_per_seed=}")
            //         if indels_per_seed[at_seed] == seed_length:
            //             at_seed += 1

            //         cost_b_chained_new = 0
            //         gap_opened = False
            //         for j in range(p):
            //             extra_cost = 0
            //             if not gap_opened and indels_per_seed[j] > 0:
            //                 extra_cost = gap_open_cost
            //                 gap_opened = True
            //             cost_b_chained_new += max(extra_cost + indel_cost * indels_per_seed[j], seed_potential)

            //         cost_b_chained = min(cost_b_chained, cost_b_chained_new)

            //     return cost_b_chained

            // def cost_crossing_p_seeds_d_diagonals(p, d, seed_potential, indel_cost, gap_open_cost, seed_length):
            //     cost_a = cost_make_all_insertions_in_one_seed(p, d, seed_potential, indel_cost, gap_open_cost)

            //     indels_per_seed, indels_left = distribute_indels_evenly(p, d)

            //     cost_b = float("inf")
            //     # Both vertical and horizontal insertions should give the same cost for d = 0
            //     if d >= 0:
            //         cost_b_while_vertical = cost_vertical_insertions(seed_potential, indel_cost, gap_open_cost, indels_per_seed, indels_left)
            //         cost_b = min(cost_b, cost_b_while_vertical)
            //     elif d <= 0:
            //         cost_b_while_horizontal = cost_horizontal_insertions(
            //             p, d, seed_potential, indel_cost, gap_open_cost, seed_length, indels_per_seed, indels_left
            //         )
            //         cost_b = min(cost_b, cost_b_while_horizontal)

            //         # We only need to try chaining all insertions together if there are insertions, seeds to put them in,
            //         # and the insertions do not over-saturate all the seeds.
            //         if d and p and 0 < abs(d) <= p * seed_length:
            //             cost_b_chained = cost_chain_horizontal_insertions(p, d, seed_potential, indel_cost, gap_open_cost, seed_length)
            //             cost_b = min(cost_b, cost_b_chained)

            //     return cost_a, cost_b
        }

        // Formula
        if formula {
            let c0 = min(max(p * r, open + extend + (p - 1) * r) + extend, open) - extend * d;
            let c1 = min(max(p * r, open + extend + (p - 1) * r - extend), p * open) + extend * d;
            let c2 = max(p * r, open + extend + (p - 1) * r);
            return max(c0, max(c1, c2));
        }

        // Insertion
        if d > 0 {
            // All insertions in 1 seed.
            let c1 = c.open + c.extend * d + (p - 1) * r;
            // Evenly distribute insertions.
            let d0 = d / p;
            let d1 = d0 + 1;
            let count_d1 = d % p;
            let count_d0 = p - count_d1;
            assert!(d0 * count_d0 + d1 * count_d1 == d);
            assert!(d1 > 0);
            let c2 = count_d0 * (if d0 == 0 { 0 } else { c.open } + c.extend * d0)
                + count_d1 * (c.open + c.extend * d1);
            return min(c1, c2);
        }

        // Deletion
        if d < 0 {
            let d = -d;
            return c.open + c.extend * d;
            // 1 insertion
            // FIXME
        }

        unreachable!();
    }
}
