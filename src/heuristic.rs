use crate::{
    increasing_function::IncreasingFunction2D,
    seeds::{find_matches, SeedMatches},
    util::*,
};

/// An object containing the settings for a heuristic.
pub trait Heuristic: std::fmt::Debug {
    type Instance: HeuristicInstance;
    fn build(&self, a: &Sequence, b: &Sequence, alphabet: &Alphabet) -> Self::Instance;
    const NAME: &'static str;
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

/// An O(1) heuristic that can be used to lower bound the distance between any two positions.
/// Used to get the distance between matches, instead of only distance to the end.
pub trait DistanceHeuristic: HeuristicInstance {
    fn h(&self, from: (Pos, Self::IncrementalState), to: (Pos, Self::IncrementalState)) -> usize;
}

// # ZERO HEURISTIC
#[derive(Debug)]
pub struct ZeroHeuristic;
impl Heuristic for ZeroHeuristic {
    type Instance = ZeroHeuristicI;
    const NAME: &'static str = "ZeroHeuristic";

    fn build(&self, _a: &Sequence, _b: &Sequence, _alphabet: &Alphabet) -> Self::Instance {
        ZeroHeuristicI
    }
}

pub struct ZeroHeuristicI;
impl HeuristicInstance for ZeroHeuristicI {
    fn h(&self, _: (Pos, Self::IncrementalState)) -> usize {
        0
    }
}

// # GAP HEURISTIC
#[derive(Debug)]
pub struct GapHeuristic;
impl Heuristic for GapHeuristic {
    type Instance = GapHeuristicI;
    const NAME: &'static str = "GapHeuristic";

    fn build(&self, a: &Sequence, b: &Sequence, _alphabet: &Alphabet) -> Self::Instance {
        GapHeuristicI {
            target: Pos(a.len(), b.len()),
        }
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

#[derive(Debug)]
pub struct CountHeuristic;
impl Heuristic for CountHeuristic {
    type Instance = CountHeuristicI;
    const NAME: &'static str = "CountHeuristic";

    fn build(&self, a: &Sequence, b: &Sequence, alphabet: &Alphabet) -> Self::Instance {
        CountHeuristicI {
            a_cnts: char_counts(a, alphabet),
            b_cnts: char_counts(b, alphabet),
        }
    }
}
pub struct CountHeuristicI {
    a_cnts: Counts,
    b_cnts: Counts,
}

impl HeuristicInstance for CountHeuristicI {
    fn h(&self, (Pos(i, j), _): (Pos, Self::IncrementalState)) -> usize {
        let mut pos = 0;
        let mut neg = 0;

        for (aold, anew, bold, bnew) in itertools::izip!(
            self.a_cnts[i],
            self.a_cnts[j],
            self.b_cnts[i],
            self.b_cnts[j],
        ) {
            let delta = (anew - aold) - (bnew - bold);
            if delta > 0 {
                pos += delta;
            } else {
                neg -= delta;
            }
        }

        max(pos, neg)
    }
}

// # SEED HEURISTIC
#[derive(Debug)]
pub struct SeedHeuristic {
    pub l: usize,
}
impl Heuristic for SeedHeuristic {
    type Instance = SeedHeuristicI;
    const NAME: &'static str = "SeedHeuristic";

    fn build(&self, a: &Sequence, b: &Sequence, alphabet: &Alphabet) -> Self::Instance {
        SeedHeuristicI::new(a, b, alphabet, self.l)
    }
}
pub struct SeedHeuristicI {
    seed_matches: SeedMatches,
    max_matches: HashMap<Pos, usize>,
}

impl SeedHeuristicI {
    fn new(a: &Sequence, b: &Sequence, alphabet: &Alphabet, l: usize) -> Self {
        let seed_matches = find_matches(a, b, alphabet, l);
        // Compute heuristic at matches.
        let mut max_matches = HashMap::new();
        max_matches.insert(Pos(a.len(), b.len()), 0);
        for pos @ Pos(i, j) in seed_matches.iter().rev() {
            // Value is 1 + max over matches bottom right of this one.
            // TODO: Make this faster.
            // TODO: Make sure seeds do not overlap.
            let val = max_matches
                .iter()
                .filter(|&(&Pos(x, y), &_)| x >= i + l && y >= j + l)
                .map(|(_, &val)| val)
                .max()
                .unwrap();
            max_matches.insert(pos, 1 + val);
        }
        SeedHeuristicI {
            seed_matches,
            max_matches,
        }
    }
}

impl HeuristicInstance for SeedHeuristicI {
    fn h(&self, (pos @ Pos(i, j), _): (Pos, Self::IncrementalState)) -> usize {
        // TODO: Find a datastructure for log-time lookup.
        let cnt = self
            .max_matches
            .iter()
            .filter(|&(&Pos(x, y), &_)| x >= i && y >= j)
            .map(|(_, &val)| val)
            .max()
            .unwrap();
        self.seed_matches.potential(pos) - cnt
    }
}

// # GAPPED SEED HEURISTIC
#[derive(Debug)]
pub struct GappedSeedHeuristic {
    pub l: usize,
}
impl Heuristic for GappedSeedHeuristic {
    type Instance = GappedSeedHeuristicI;
    const NAME: &'static str = "GappedSeedHeuristic";

    fn build(&self, a: &Sequence, b: &Sequence, alphabet: &Alphabet) -> Self::Instance {
        GappedSeedHeuristicI::new(a, b, alphabet, self.l)
    }
}
pub struct GappedSeedHeuristicI {
    seed_matches: SeedMatches,
    h_map: HashMap<Pos, isize>,
}

impl GappedSeedHeuristicI {
    fn new(a: &Sequence, b: &Sequence, alphabet: &Alphabet, l: usize) -> Self {
        let seed_matches = find_matches(a, b, alphabet, l);
        let skipped: &mut usize = &mut 0;

        let mut h_map = HashMap::new();
        h_map.insert(Pos(a.len(), b.len()), 0);
        for pos @ Pos(i, j) in seed_matches.iter().rev() {
            let update_val = h_map
                .iter()
                .filter(|&(&Pos(x, y), &_)| x >= i + l && y >= j + l)
                .map(|(&Pos(x, y), &val)| val + abs_diff(x - i, y - j) as isize - 1)
                .min()
                .unwrap();
            let query_val = h_map
                .iter()
                .filter(|&(&Pos(x, y), &_)| x >= i && y >= j)
                .map(|(&Pos(x, y), &val)| val + abs_diff(x - i, y - j) as isize)
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
        GappedSeedHeuristicI {
            seed_matches,
            h_map,
        }
    }
}
impl HeuristicInstance for GappedSeedHeuristicI {
    fn h(&self, (pos @ Pos(i, j), _): (Pos, Self::IncrementalState)) -> usize {
        (self.seed_matches.potential(pos) as isize
            + self
                .h_map
                .iter()
                .filter(|&(&Pos(x, y), &_)| x >= i && y >= j)
                .map(|(&Pos(x, y), &val)| {
                    // TODO: Should there be a +- 1 here? Or take into account
                    // whether the current position/column is a match?
                    val + abs_diff(x - i, y - j) as isize
                })
                .min()
                .unwrap()) as usize
    }
}

// # FAST SEED HEURISTIC
#[derive(Debug)]
pub struct FastSeedHeuristic {
    pub l: usize,
}
impl Heuristic for FastSeedHeuristic {
    type Instance = FastSeedHeuristicI;
    const NAME: &'static str = "FastSeedHeuristic";

    fn build(&self, a: &Sequence, b: &Sequence, alphabet: &Alphabet) -> Self::Instance {
        println!("build");
        let x = FastSeedHeuristicI::new(a, b, alphabet, self.l);
        println!("building done");
        x
    }
}
pub struct FastSeedHeuristicI {
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
            target: Pos(a.len(), b.len()),
            f,
        }
    }

    fn invert_pos(&self, Pos(i, j): Pos) -> Pos {
        Pos(self.target.0 - i, self.target.1 - j)
    }
}
impl HeuristicInstance for FastSeedHeuristicI {
    fn h(&self, (_pos, parent): (Pos, Self::IncrementalState)) -> usize {
        self.f.val(parent)
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
