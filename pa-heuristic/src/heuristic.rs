pub mod bruteforce_gcsh;
pub mod csh;
pub mod distances;
pub mod sh;
pub mod wrappers;

use crate::prelude::*;
use crate::seeds::Seed;
use crate::{contour::Arrow, matches::*};
use clap::ValueEnum;
use derive_more::AddAssign;
use serde::{Deserialize, Serialize};

pub use bruteforce_gcsh::*;
pub use csh::*;
pub use distances::*;
pub use sh::*;

#[derive(Debug, ValueEnum, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Prune {
    None,
    Start,
    End,
    Both,
}
impl Prune {
    pub fn is_enabled(&self) -> bool {
        match self {
            Prune::None => false,
            _ => true,
        }
    }
    pub fn start(&self) -> bool {
        match self {
            Prune::None | Prune::End => false,
            Prune::Start | Prune::Both => true,
        }
    }
    pub fn end(&self) -> bool {
        match self {
            Prune::None | Prune::Start => false,
            Prune::End | Prune::Both => true,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Pruning {
    pub enabled: Prune,
    /// Skip pruning one in N.
    pub skip_prune: Option<usize>,
}

impl Default for Pruning {
    fn default() -> Self {
        Self::start()
    }
}

impl Pruning {
    pub fn new(enabled: Prune) -> Self {
        Self {
            enabled,
            skip_prune: None,
        }
    }
    pub fn disabled() -> Self {
        Pruning {
            enabled: Prune::None,
            skip_prune: None,
        }
    }
    pub fn start() -> Self {
        Pruning {
            enabled: Prune::Start,
            skip_prune: None,
        }
    }
    pub fn both() -> Self {
        Pruning {
            enabled: Prune::Both,
            skip_prune: None,
        }
    }

    pub fn is_enabled(&self) -> bool {
        match self.enabled {
            Prune::None => false,
            _ => true,
        }
    }
    pub fn prune_start(&self) -> bool {
        match self.enabled {
            Prune::None | Prune::End => false,
            Prune::Start | Prune::Both => true,
        }
    }
    pub fn prune_end(&self) -> bool {
        match self.enabled {
            Prune::None | Prune::Start => false,
            Prune::End | Prune::Both => true,
        }
    }
}

#[derive(Clone, AddAssign, Default, Copy, Debug)]
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

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>) -> Self::Instance<'a>;

    // Heuristic properties.
    fn name(&self) -> String;
}

pub trait PosOrderT: PartialOrd + Default + Copy + std::fmt::Debug {
    fn from_pos(p: Pos) -> Self;
    fn max(p: Self, q: Self) -> Self;
    type D: std::fmt::Debug;
    fn tip_start(s: usize, p: Self) -> Self;
}

impl PosOrderT for () {
    fn from_pos(_: Pos) -> Self {}
    fn max(_: Self, _: Self) -> Self {}
    type D = ();
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
    fn tip_start(s: usize, p: Self) -> Self {
        p.saturating_sub(s as I)
    }
}

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
            .map_or(false, |sm| sm.seeds.is_seed_start_or_end(pos))
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

    /// A descriptive string of the heuristic settings, used for failing assertions.
    fn params_string(&self) -> String {
        "".into()
    }
}
