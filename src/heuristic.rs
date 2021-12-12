use serde::Serialize;

use crate::{
    increasing_function::IncreasingFunction2D,
    seeds::{find_matches, Match, SeedMatches},
    util::*,
};

#[derive(Serialize)]
pub struct HeuristicParams {
    pub heuristic: String,
    pub distance_function: Option<String>,
    pub l: Option<usize>,
    pub match_distance: Option<usize>,
    pub pruning: Option<bool>,
}

/// An object containing the settings for a heuristic.
pub trait Heuristic: std::fmt::Debug + Copy {
    type Instance: HeuristicInstance;

    // Heuristic properties.
    fn name(&self) -> &'static str;
    fn l(&self) -> Option<usize> {
        None
    }
    fn match_distance(&self) -> Option<usize> {
        None
    }
    fn pruning(&self) -> Option<bool> {
        None
    }
    fn distance(&self) -> Option<&'static str> {
        None
    }

    fn build(&self, a: &Sequence, b: &Sequence, alphabet: &Alphabet) -> Self::Instance;

    fn params(&self) -> HeuristicParams {
        HeuristicParams {
            heuristic: self.name().to_string(),
            distance_function: self.distance().map(|x| x.to_string()),
            l: self.l(),
            match_distance: self.match_distance(),
            pruning: self.pruning(),
        }
    }
}

/// An instantiation of a heuristic for a specific pair of sequences.
pub trait HeuristicInstance {
    fn h(&self, pos: (Pos, Self::IncrementalState)) -> usize;
    fn expand(&mut self, _pos: Pos) {}

    // TODO: Simplify this, and just use a map inside the heuristic.
    type IncrementalState: std::hash::Hash + Eq + Copy + Default = ();
    fn incremental_h(
        &self,
        _parent: (Pos, Self::IncrementalState),
        _pos: Pos,
    ) -> Self::IncrementalState {
        Default::default()
    }
    fn root_state(&self) -> Self::IncrementalState {
        Default::default()
    }

    // Some statistics of the heuristic.
    fn num_seeds(&self) -> Option<usize> {
        None
    }
    fn matches(&self) -> Option<&Vec<Match>> {
        None
    }
    fn num_matches(&self) -> Option<usize> {
        None
    }
}

/// An O(1) evaluation heuristic that can be used to lower bound the distance between any two positions.
/// Used to get the distance between matches, instead of only distance to the end.
// TODO: Can we get away with only one of these two traits?
pub trait DistanceHeuristic: Heuristic {
    // TODO: Provide default implementations for these.
    type DistanceInstance: DistanceHeuristicInstance;
    fn build(&self, a: &Sequence, b: &Sequence, alphabet: &Alphabet) -> Self::DistanceInstance;
}

pub trait DistanceHeuristicInstance: HeuristicInstance {
    fn distance(&self, from: Pos, to: Pos) -> usize;
}

// # ZERO HEURISTIC
#[derive(Debug, Clone, Copy)]
pub struct ZeroHeuristic;
impl Heuristic for ZeroHeuristic {
    type Instance = ZeroHeuristicI;

    fn name(&self) -> &'static str {
        "Zero"
    }

    fn build(&self, _a: &Sequence, _b: &Sequence, _alphabet: &Alphabet) -> Self::Instance {
        ZeroHeuristicI
    }
}
impl DistanceHeuristic for ZeroHeuristic {
    type DistanceInstance = ZeroHeuristicI;

    fn build(&self, a: &Sequence, b: &Sequence, alphabet: &Alphabet) -> Self::DistanceInstance {
        ZeroHeuristicI
    }
}

pub struct ZeroHeuristicI;
impl HeuristicInstance for ZeroHeuristicI {
    fn h(&self, _: (Pos, Self::IncrementalState)) -> usize {
        0
    }
}
impl DistanceHeuristicInstance for ZeroHeuristicI {
    fn distance(&self, _from: Pos, _to: Pos) -> usize {
        0
    }
}

// # GAP HEURISTIC
#[derive(Debug, Clone, Copy)]
pub struct GapHeuristic;
impl Heuristic for GapHeuristic {
    type Instance = GapHeuristicI;
    fn name(&self) -> &'static str {
        "Gap"
    }

    fn build(&self, a: &Sequence, b: &Sequence, _alphabet: &Alphabet) -> Self::Instance {
        GapHeuristicI {
            target: Pos(a.len(), b.len()),
        }
    }
}
impl DistanceHeuristic for GapHeuristic {
    type DistanceInstance = GapHeuristicI;

    fn build(&self, a: &Sequence, b: &Sequence, alphabet: &Alphabet) -> Self::DistanceInstance {
        <GapHeuristic as Heuristic>::build(self, a, b, alphabet)
    }
}
pub struct GapHeuristicI {
    target: Pos,
}

impl HeuristicInstance for GapHeuristicI {
    fn h(&self, (Pos(i, j), _): (Pos, Self::IncrementalState)) -> usize {
        abs_diff(self.target.0 - i, self.target.1 - j)
    }
}
impl DistanceHeuristicInstance for GapHeuristicI {
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
    type Instance = CountHeuristicI;
    fn name(&self) -> &'static str {
        "Count"
    }

    fn build(&self, a: &Sequence, b: &Sequence, alphabet: &Alphabet) -> Self::Instance {
        CountHeuristicI {
            a_cnts: char_counts(a, alphabet),
            b_cnts: char_counts(b, alphabet),
            target: Pos(a.len(), b.len()),
        }
    }
}
impl DistanceHeuristic for CountHeuristic {
    type DistanceInstance = CountHeuristicI;

    fn build(&self, a: &Sequence, b: &Sequence, alphabet: &Alphabet) -> Self::DistanceInstance {
        <CountHeuristic as Heuristic>::build(self, a, b, alphabet)
    }
}
pub struct CountHeuristicI {
    a_cnts: Counts,
    b_cnts: Counts,
    target: Pos,
}

impl HeuristicInstance for CountHeuristicI {
    fn h(&self, (pos, _): (Pos, Self::IncrementalState)) -> usize {
        self.distance(pos, self.target)
    }
}

impl DistanceHeuristicInstance for CountHeuristicI {
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
    type Instance = BiCountHeuristicI;
    fn name(&self) -> &'static str {
        "BiCount"
    }

    fn build(&self, a: &Sequence, b: &Sequence, alphabet: &Alphabet) -> Self::Instance {
        BiCountHeuristicI {
            cnt: DistanceHeuristic::build(&CountHeuristic, a, b, alphabet),
            a_cnts: char_bicounts(a, alphabet),
            b_cnts: char_bicounts(b, alphabet),
            target: Pos(a.len(), b.len()),
        }
    }
}
impl DistanceHeuristic for BiCountHeuristic {
    type DistanceInstance = BiCountHeuristicI;

    fn build(&self, a: &Sequence, b: &Sequence, alphabet: &Alphabet) -> Self::DistanceInstance {
        <BiCountHeuristic as Heuristic>::build(self, a, b, alphabet)
    }
}
pub struct BiCountHeuristicI {
    cnt: CountHeuristicI,
    a_cnts: BiCounts,
    b_cnts: BiCounts,
    target: Pos,
}

impl HeuristicInstance for BiCountHeuristicI {
    fn h(&self, (pos, _): (Pos, Self::IncrementalState)) -> usize {
        self.distance(pos, self.target)
    }
}

impl DistanceHeuristicInstance for BiCountHeuristicI {
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

// # SEED HEURISTIC
#[derive(Debug, Clone, Copy)]
pub struct SeedHeuristic<DH: DistanceHeuristic> {
    pub l: usize,
    pub match_distance: usize,
    pub distance_function: DH,
    pub pruning: bool,
}
impl<DH: DistanceHeuristic> Heuristic for SeedHeuristic<DH> {
    type Instance = SeedHeuristicI<DH>;

    fn build(&self, a: &Sequence, b: &Sequence, alphabet: &Alphabet) -> Self::Instance {
        SeedHeuristicI::new(a, b, alphabet, &self)
    }
    fn l(&self) -> Option<usize> {
        Some(self.l)
    }
    fn match_distance(&self) -> Option<usize> {
        Some(self.match_distance)
    }
    fn pruning(&self) -> Option<bool> {
        Some(self.pruning)
    }
    fn distance(&self) -> Option<&'static str> {
        Some(self.distance_function.name())
    }
    fn name(&self) -> &'static str {
        "Seed"
    }
}
pub struct SeedHeuristicI<DH: DistanceHeuristic> {
    seed_matches: SeedMatches,
    h_map: HashMap<Pos, usize>,
    distance_function: DH::DistanceInstance,
    target: Pos,
    // TODO: Replace this by params: SeedHeuristic
    pruning: bool,
    match_distance: usize,
}

impl<DH: DistanceHeuristic> SeedHeuristicI<DH> {
    fn new(a: &Sequence, b: &Sequence, alphabet: &Alphabet, params: &SeedHeuristic<DH>) -> Self {
        let seed_matches = find_matches(a, b, alphabet, params.l, params.match_distance);
        let skipped: &mut usize = &mut 0;

        let distance_function = DistanceHeuristic::build(&params.distance_function, a, b, alphabet);

        let mut h_map = HashMap::new();
        h_map.insert(Pos(a.len(), b.len()), 0);
        for Match {
            start,
            end,
            match_distance,
        } in seed_matches.iter().rev()
        {
            let update_val = h_map
                .iter()
                .filter(|&(parent, _)| parent >= end)
                .map(|(&parent, &val)| {
                    val + max(
                        distance_function.distance(*start, parent),
                        seed_matches.potential(*start) - seed_matches.potential(parent)
                            + match_distance
                            - (params.match_distance + 1),
                    )
                })
                .min()
                .unwrap();
            let query_val = h_map
                .iter()
                .filter(|&(parent, _)| parent >= start)
                .map(|(&parent, &val)| -> usize {
                    val + max(
                        distance_function.distance(*start, parent),
                        seed_matches.potential(*start) - seed_matches.potential(parent),
                    )
                })
                .min()
                .unwrap();

            // TODO: Report number of inserted and skipped matches
            if update_val < query_val {
                h_map.insert(*start, update_val);
            } else {
                *skipped += 1;
            }
        }
        SeedHeuristicI {
            seed_matches,
            h_map,
            distance_function,
            target: Pos(a.len(), b.len()),
            pruning: params.pruning,
            match_distance: params.match_distance,
        }
    }
}

impl<DH: DistanceHeuristic> HeuristicInstance for SeedHeuristicI<DH> {
    fn h(&self, (pos, _): (Pos, Self::IncrementalState)) -> usize {
        self.h_map
            .iter()
            .filter(|&(&parent, &_)| parent >= pos)
            .map(|(&parent, &val)| {
                val + max(
                    self.distance_function.distance(pos, parent),
                    self.seed_matches.potential(pos) - self.seed_matches.potential(parent),
                )
            })
            .min()
            .unwrap() as usize
    }
    fn num_seeds(&self) -> Option<usize> {
        Some(self.seed_matches.num_seeds)
    }
    fn matches(&self) -> Option<&Vec<Match>> {
        Some(&self.seed_matches.matches)
    }
    fn num_matches(&self) -> Option<usize> {
        Some(self.seed_matches.matches.len())
    }
    fn expand(&mut self, pos: Pos) {
        if !self.pruning {
            return;
        }
        // If this is a matching position, rebuild the heuristic.
        if self.h_map.remove(&pos).is_none() {
            return;
        }

        let mut h_map = HashMap::new();
        h_map.insert(self.target, 0);
        for Match {
            start,
            end,
            match_distance,
        } in self.seed_matches.matches.iter().rev()
        {
            if !self.h_map.contains_key(&pos) {
                continue;
            }

            let update_val = h_map
                .iter()
                .filter(|&(parent, _)| parent >= end)
                .map(|(&parent, &val)| {
                    val + max(
                        self.distance_function.distance(*start, parent),
                        self.seed_matches.potential(*start) - self.seed_matches.potential(parent)
                            + match_distance
                            - (self.match_distance + 1),
                    )
                })
                .min()
                .unwrap();
            let query_val = h_map
                .iter()
                .filter(|&(parent, _)| parent >= start)
                .map(|(&parent, &val)| {
                    val + max(
                        self.distance_function.distance(*start, parent),
                        self.seed_matches.potential(*start) - self.seed_matches.potential(parent),
                    )
                })
                .min()
                .unwrap();

            // TODO: Report number of inserted and skipped matches
            if update_val < query_val {
                h_map.insert(*start, update_val);
            }
        }
    }
}

/*
// # FAST SEED HEURISTIC
// TODO: Make this work for the other distance functions.
// TODO: Inherit this from SeedHeuristic
#[derive(Debug, Clone, Copy)]
pub struct FastSeedHeuristic {
    pub l: usize,
}
impl Heuristic for FastSeedHeuristic {
    type Instance = FastSeedHeuristicI;
    fn name(&self) -> &'static str {
        "FastSeed"
    }

    fn build(&self, a: &Sequence, b: &Sequence, alphabet: &Alphabet) -> Self::Instance {
        FastSeedHeuristicI::new(a, b, alphabet, self.l)
    }
    fn l(&self) -> Option<usize> {
        Some(self.l)
    }
    fn distance(&self) -> Option<&'static str> {
        Some("Zero")
    }
}
pub struct FastSeedHeuristicI {
    seed_matches: SeedMatches,
    target: Pos,
    f: IncreasingFunction2D<usize>,
}

impl FastSeedHeuristicI {
    pub fn new(a: &Sequence, b: &Sequence, alphabet: &Alphabet, l: usize) -> Self {
        let seed_matches = find_matches(a, b, alphabet, l);

        // The increasing function goes back from the end, and uses (0,0) for the final state.
        let f = IncreasingFunction2D::new(
            seed_matches
                .iter()
                .rev()
                .map(|Pos(i, j)| Pos(a.len() - i, b.len() - j)),
            l,
        );

        FastSeedHeuristicI {
            seed_matches,
            target: Pos(a.len(), b.len()),
            f,
        }
    }

    fn invert_pos(&self, Pos(i, j): Pos) -> Pos {
        Pos(self.target.0 - i, self.target.1 - j)
    }
}
impl HeuristicInstance for FastSeedHeuristicI {
    fn h(&self, (pos, parent): (Pos, Self::IncrementalState)) -> usize {
        self.seed_matches.potential(pos) - self.f.val(parent)
    }

    type IncrementalState = crate::increasing_function::NodeIndex;

    fn incremental_h(
        &self,
        parent: (Pos, Self::IncrementalState),
        pos: Pos,
    ) -> Self::IncrementalState {
        // We can unwrap because (0,0) is part of the map.
        self.f.get_jump(self.invert_pos(pos), parent.1).unwrap()
    }

    fn root_state(&self) -> Self::IncrementalState {
        self.f.root()
    }
    fn num_seeds(&self) -> Option<usize> {
        Some(self.seed_matches.potential(Pos(0, 0)))
    }
    fn matches(&self) -> Option<Vec<Pos>> {
        Some(self.seed_matches.iter().collect())
    }
    fn num_matches(&self) -> Option<usize> {
        Some(self.seed_matches.num_matches())
    }
}
*/
