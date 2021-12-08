use crate::{
    increasing_function::IncreasingFunction2D,
    seeds::{find_matches, SeedMatches},
    util::*,
};

/// An object containing the settings for a heuristic.
pub trait Heuristic: std::fmt::Debug + Copy {
    type Instance: HeuristicInstance;
    fn build(&self, a: &Sequence, b: &Sequence, alphabet: &Alphabet) -> Self::Instance;
}

/// An instantiation of a heuristic for a specific pair of sequences.
pub trait HeuristicInstance {
    fn h(&self, pos: (Pos, Self::IncrementalState)) -> usize;

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
}

/// An O(1) evaluation heuristic that can be used to lower bound the distance between any two positions.
/// Used to get the distance between matches, instead of only distance to the end.
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
    let mut counts = Counts::with_capacity(a.len() + 1);
    counts.push([0, 0, 0, 0]);
    for ch in transform.transform(a) {
        counts.push(*counts.last().unwrap());
        counts.last_mut().unwrap()[ch as usize] += 1;
    }
    counts
}

#[derive(Debug, Clone, Copy)]
pub struct CountHeuristic;
impl Heuristic for CountHeuristic {
    type Instance = CountHeuristicI;

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

// # SEED HEURISTIC
#[derive(Debug, Clone, Copy)]
pub struct SeedHeuristic<DH: DistanceHeuristic> {
    pub l: usize,
    pub distance: DH,
}
impl<DH: DistanceHeuristic> Heuristic for SeedHeuristic<DH> {
    type Instance = SeedHeuristicI<DH>;

    fn build(&self, a: &Sequence, b: &Sequence, alphabet: &Alphabet) -> Self::Instance {
        SeedHeuristicI::new(a, b, alphabet, self.l, self.distance)
    }
}
pub struct SeedHeuristicI<DH: DistanceHeuristic> {
    seed_matches: SeedMatches,
    h_map: HashMap<Pos, isize>,
    distance: DH::DistanceInstance,
}

impl<DH: DistanceHeuristic> SeedHeuristicI<DH> {
    fn new(a: &Sequence, b: &Sequence, alphabet: &Alphabet, l: usize, distance: DH) -> Self {
        let seed_matches = find_matches(a, b, alphabet, l);
        let skipped: &mut usize = &mut 0;

        let distance = DistanceHeuristic::build(&distance, a, b, alphabet);

        let mut h_map = HashMap::new();
        h_map.insert(Pos(a.len(), b.len()), 0);
        for pos @ Pos(i, j) in seed_matches.iter().rev() {
            let update_val = h_map
                .iter()
                .filter(|&(&Pos(x, y), &_)| x >= i + l && y >= j + l)
                .map(|(&to, &val)| val + distance.distance(pos, to) as isize - 1)
                .min()
                .unwrap();
            let query_val = h_map
                .iter()
                .filter(|&(&Pos(x, y), &_)| x >= i && y >= j)
                .map(|(&to, &val)| val + distance.distance(pos, to) as isize)
                .min()
                .unwrap();

            if update_val < query_val {
                h_map.insert(pos, update_val);
            } else {
                *skipped += 1;
            }
            //println!("{:?} => {}", pos, val);
        }
        //println!("Skipped matches: {}", skipped);
        SeedHeuristicI {
            seed_matches,
            h_map,
            distance,
        }
    }
}

impl<DH: DistanceHeuristic> HeuristicInstance for SeedHeuristicI<DH> {
    fn h(&self, (pos @ Pos(i, j), _): (Pos, Self::IncrementalState)) -> usize {
        let min = self
            .h_map
            .iter()
            .filter(|&(&Pos(x, y), &_)| x >= i && y >= j)
            .map(|(&to, &val)| val + self.distance.distance(pos, to) as isize)
            .min()
            .unwrap();
        let potential = self.seed_matches.potential(pos);
        let v = (potential as isize + min) as usize;
        //println!("{:?} -> {}", pos, v);
        v
    }
}

// # FAST SEED HEURISTIC
#[derive(Debug, Clone, Copy)]
pub struct FastSeedHeuristic {
    pub l: usize,
}
impl Heuristic for FastSeedHeuristic {
    type Instance = FastSeedHeuristicI;

    fn build(&self, a: &Sequence, b: &Sequence, alphabet: &Alphabet) -> Self::Instance {
        FastSeedHeuristicI::new(a, b, alphabet, self.l)
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
        let v = self.seed_matches.potential(pos) - self.f.val(parent);
        //println!("{:?} -> {}", pos, v);
        v
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
}

// MERGED, FOR TESTING THEY ARE EQUAL.
#[derive(Debug, Clone, Copy)]
pub struct MergedSeedHeuristic {
    pub l: usize,
}
impl Heuristic for MergedSeedHeuristic {
    type Instance = MergedSeedHeuristicI;

    fn build(&self, a: &Sequence, b: &Sequence, alphabet: &Alphabet) -> Self::Instance {
        MergedSeedHeuristicI::new(a, b, alphabet, self.l)
    }
}
pub struct MergedSeedHeuristicI {
    seed: SeedHeuristicI<ZeroHeuristic>,
    fast: FastSeedHeuristicI,
}

impl MergedSeedHeuristicI {
    pub fn new(a: &Sequence, b: &Sequence, alphabet: &Alphabet, l: usize) -> Self {
        MergedSeedHeuristicI {
            seed: SeedHeuristicI::new(a, b, alphabet, l, ZeroHeuristic),
            fast: FastSeedHeuristicI::new(a, b, alphabet, l),
        }
    }
}
impl HeuristicInstance for MergedSeedHeuristicI {
    fn h(&self, x: (Pos, Self::IncrementalState)) -> usize {
        let a = self.seed.h((x.0, ()));
        let b = self.fast.h(x);
        assert_eq!(a, b, "Values differ at {:?}: {} vs {}", x, a, b);
        a
    }

    type IncrementalState = crate::increasing_function::NodeIndex;

    fn incremental_h(
        &self,
        parent: (Pos, Self::IncrementalState),
        pos: Pos,
    ) -> Self::IncrementalState {
        self.fast.incremental_h(parent, pos)
    }

    fn root_state(&self) -> Self::IncrementalState {
        self.fast.root_state()
    }
}
