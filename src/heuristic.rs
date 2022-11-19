pub mod bruteforce_csh;
pub mod chained_seed;
pub mod distance;
pub mod equal;
pub mod max;
pub mod mirror;
pub mod path;
pub mod perfect;
pub mod seed;
pub mod symmetric;

use crate::{matches::Match, prelude::*};

pub use bruteforce_csh::*;
pub use chained_seed::*;
use derive_more::AddAssign;
pub use distance::*;
pub use equal::*;
pub use max::*;
pub use mirror::*;
pub use perfect::*;
pub use seed::*;
pub use symmetric::*;

#[derive(Default, Clone)]
pub struct HeuristicParams {
    pub name: String,
    pub distance_function: String,
    pub k: I,
    pub max_match_cost: MatchCost,
    pub pruning: Pruning,
}

#[derive(Clone, AddAssign, Default, Copy)]
pub struct HeuristicStats {
    pub num_seeds: I,
    pub num_matches: usize,
    pub num_filtered_matches: usize,
    pub pruning_duration: f32,
    pub num_pruned: usize,
    pub h0: Cost,
    pub h0_end: Cost,
    pub prune_count: usize,
}

/// An object containing the settings for a heuristic.
pub trait Heuristic: std::fmt::Debug + Copy {
    type Instance<'a>: HeuristicInstance<'a>;
    const IS_DEFAULT: bool = false;

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>) -> Self::Instance<'a> {
        self.build_with_filter(a, b, |_, _| false)
    }
    fn build_with_filter<'a>(
        &self,
        _a: Seq<'a>,
        _b: Seq<'a>,
        _f: impl FnMut(&Match, Cost) -> bool,
    ) -> Self::Instance<'a> {
        unimplemented!();
    }

    // Heuristic properties.
    fn name(&self) -> String;

    fn params(&self) -> HeuristicParams {
        HeuristicParams {
            name: self.name(),
            ..Default::default()
        }
    }
}

pub trait PosOrderT: PartialOrd + Default + Copy + std::fmt::Debug {
    fn from_pos(p: Pos) -> Self;
    fn max(p: Self, q: Self) -> Self;
    type D: std::fmt::Debug;
    fn diff(p: Self, q: Self) -> Self::D;
    fn tip_start(s: usize, p: Self) -> Self;
}

impl PosOrderT for () {
    fn from_pos(_: Pos) -> Self {}
    fn max(_: Self, _: Self) -> Self {}
    type D = ();
    fn diff(_: Self, _: Self) -> Self::D {}
    fn tip_start(_: usize, _: Self) -> Self {}
}

/// The order for CSH
impl PosOrderT for Pos {
    fn from_pos(p: Pos) -> Self {
        p
    }
    fn max(p: Self, q: Self) -> Self {
        Pos(max(p.0, q.0), max(p.1, q.1))
    }
    type D = (i32, i32);
    fn diff(p: Self, q: Self) -> Self::D {
        (p.0 as i32 - q.0 as i32, p.1 as i32 - q.1 as i32)
    }
    fn tip_start(s: usize, p: Self) -> Self {
        Pos(p.0.saturating_sub(s as I), p.1.saturating_sub(s as I))
    }
}

/// The order of SH.
impl PosOrderT for I {
    fn from_pos(p: Pos) -> Self {
        p.0
    }
    fn max(p: Self, q: Self) -> Self {
        max(p, q)
    }
    type D = i32;
    fn diff(p: Self, q: Self) -> Self::D {
        p as i32 - q as i32
    }
    fn tip_start(s: usize, p: Self) -> Self {
        p.saturating_sub(s as I)
    }
}

pub type X = ();

/// An instantiation of a heuristic for a specific pair of sequences.
pub trait HeuristicInstance<'a> {
    fn h(&self, pos: Pos) -> Cost;

    /// The internal contour value at the given position, if available.
    fn layer(&self, _pos: Pos) -> Option<Cost> {
        None
    }

    /// The internal contour value at the given position, if available.
    fn layer_with_hint(&self, _pos: Pos, _hint: Self::Hint) -> Option<(Cost, Self::Hint)> {
        None
    }

    fn h_with_parent(&self, pos: Pos) -> (Cost, Pos) {
        (self.h(pos), Pos::default())
    }

    type Hint: Copy + Default + std::fmt::Debug = ();
    fn h_with_hint(&self, pos: Pos, _hint: Self::Hint) -> (Cost, Self::Hint) {
        (self.h(pos), Default::default())
    }

    /// FIXME: DELETE THIS FUNCTION.
    fn root_state(&self, _root_pos: Pos) -> Self::Hint {
        Default::default()
    }

    fn root_potential(&self) -> Cost {
        0
    }

    /// The seed matches used by the heuristic.
    fn seed_matches(&self) -> Option<&SeedMatches> {
        None
    }

    /// A* will checked for consistency whenever this returns true.
    fn is_seed_start_or_end(&self, pos: Pos) -> bool {
        self.seed_matches()
            .map_or(false, |sm| sm.is_seed_start_or_end(pos))
    }

    type Order: PosOrderT = ();

    /// Returns the offset by which all expanded states in the priority queue can be shifted.
    ///
    /// `seed_cost`: The cost made in the seed ending at pos.
    fn prune(&mut self, _pos: Pos, _hint: Self::Hint) -> (Cost, Self::Order) {
        (0, Default::default())
    }

    /// Tells the heuristic that the position was explored, so it knows which
    /// positions need to be updated when propagating the pruning to the
    /// priority queue.
    fn explore(&mut self, _pos: Pos) {}

    fn stats(&mut self) -> HeuristicStats {
        Default::default()
    }

    fn matches(&self) -> Option<Vec<Match>> {
        None
    }

    fn seeds(&self) -> Option<&Vec<Seed>> {
        None
    }

    fn params_string(&self) -> String {
        "".into()
    }
}
