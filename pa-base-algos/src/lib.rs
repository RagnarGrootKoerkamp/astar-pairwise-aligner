#![feature(let_chains, step_trait, int_roundings, portable_simd)]

use pa_types::Cost;
use serde::{Deserialize, Serialize};
use std::cmp::{max, min};

mod edit_graph;
mod front;

pub mod cli;
pub mod dt;
pub mod nw;

// #[cfg(test)]
// mod tests;

const PRINT: bool = false;

/// Find the cost using exponential search based on `f`.
///
/// Tries values `offset + s0 * f^i`.
///
/// * Worst case growth factor analysis
///
/// 1, g, g^2, ...
///
/// worst-case overshoot: g^k = g*s
/// Assuming O(ng) work per guess (Gap, GapGap)
///   n(1+g+...+g^k) = n*(g*g^k-1)/(g-1) = n*(g^2 s-1)/(g-1) ~ ns g^2/(g-1)
///   minimize g^2/(g-1):
///   derivative 0: 0 = (2g (g-1) - g^2) / (g-1)^2 => 0 = g^2-2g = g(g-2)
/// g=2
/// 4ns
///
/// Assuming O(g^2) work per guess (Dijkstra, Astar(GapCost), when errors are uniform)
///   1 + g^2 + g^4 + ... + g^2k ~ g^{2k+2} / (g^2-1) = ns g^4 / (g^2-1)
///   minimize g^4/(g^2-1)
///   derivative 0: 0 = 4g^3(g^2-1) - g^4 2g = 2g^5 - 4g^3 = 2 g^3 (g^2-2)
/// g=sqrt(2)
/// 2ns
/// in case all errors are at the end and runtime is O(ng) per guess:
/// 4.8 ns, only slightly worse than 4ns.
///
/// Assuming O(g^2) work per guess (Dijkstra, Astar(GapCost), when errors are uniform)
/// * ALSO ASSUMING THAT OVERSHOOT IS ONLY O(ng) cost.
/// * TODO: Verify this
///   1 + g^2 + g^4 + ... + g^{2k-2} + n g^k ~ g^{2k} / (g^2-1) + n g^k = ns g^2 / (g^2-1) + ns g
///   minimize g^2/(g^2-1) + g = (g^3+g^2-g)/(g^2-1)
///   derivative 0: 0 = 4g^3(g^2-1) - g^4 2g = 2g^5 - 4g^3 = 2 g^3 (g^2-2)
/// g=sqrt(2)
/// 2ns
/// in case all errors are at the end and runtime is O(ng) per guess:
/// 4.8 ns, only slightly worse than 4ns.
fn exponential_search<T>(
    offset: Cost,
    s0: Cost,
    factor: f32,
    mut f: impl FnMut(Cost) -> Option<(Cost, T)>,
) -> (Cost, T) {
    let mut last_s = -1;
    let mut s = offset + s0;
    let mut maxs = Cost::MAX;
    // TODO: Fix the potential infinite loop here.
    //
    // Sanity checks:
    // - Once the answer is found, this should be larger than all previous thresholds.
    // - Once a value for maxs has been found, all subsequent larger values of s
    //   should return a value that is smaller.
    loop {
        if let Some((cost, t)) = f(s) {
            assert!(
                cost <= maxs,
                "A solution {maxs} was found for a previous s<={last_s}, but s={s} gives {cost}"
            );
            if cost <= s {
                assert!(cost > last_s, "Cost {cost} was found at s {s} but should already have been found at last_s {last_s}");
                return (cost, t);
            } else {
                // If some value was returned this is an upper bound on the answer.
                maxs = min(maxs, cost);
            }
        } else {
            assert!(
                maxs == Cost::MAX,
                "A solution {maxs} was found for a previous s<={last_s}, but not for current s={s}"
            );
        }
        last_s = s;
        s = max((factor * (s - offset) as f32).ceil() as Cost, 1) + offset;
        s = min(s, maxs);
    }
}

fn linear_search<T>(
    s0: Cost,
    delta: Cost,
    mut f: impl FnMut(Cost) -> Option<(Cost, T)>,
) -> (Cost, T) {
    let mut last_s = -1;
    let mut s = s0;
    let mut maxs = Cost::MAX;
    // TODO: Fix the potential infinite loop here.
    //
    // Sanity checks:
    // - Once the answer is found, this should be larger than all previous thresholds.
    // - Once a value for maxs has been found, all subsequent larger values of s
    //   should return a value that is smaller.
    loop {
        if let Some((cost, t)) = f(s) {
            assert!(
                cost <= maxs,
                "A solution {maxs} was found for a previous s<={last_s}, but s={s} gives {cost}"
            );
            if cost <= s {
                assert!(cost > last_s, "Cost {cost} was found at s {s} but should already have been found at last_s {last_s}");
                return (cost, t);
            } else {
                // If some value was returned this is an upper bound on the answer.
                maxs = min(maxs, cost);
            }
        } else {
            assert!(
                maxs == Cost::MAX,
                "A solution {maxs} was found for a previous s<={last_s}, but not for current s={s}"
            );
        }
        last_s = s;
        s = min(s + delta, maxs);
    }
}

use pa_heuristic::{GapCost, NoCost};

/// Enum for the various computational domain types.
/// See Ukkonen, Scrooge, O(NP), Papamichail, A*PA
///
/// Distance from start can be none, gap, or g*
/// Distance to end can be none, gap, h
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Domain<H> {
    /// Compute the entire rectangle
    Full,
    /// States with gap(s, u) <= f
    GapStart,
    /// States with gap(s, u) + gap(u, t) <= f
    GapGap,
    /// States with g(u) + h(u) <= f, for some arbitrary h.
    /// For Dijkstra, use H=NoCost.
    /// For GapCost to end, use H=GapCost.
    Astar(H),
}

use Domain::*;

impl Default for Domain<()> {
    fn default() -> Self {
        Astar(())
    }
}

impl Domain<()> {
    pub fn into(self) -> Domain<NoCost> {
        match self {
            Full => Full,
            GapStart => GapStart,
            GapGap => GapGap,
            Astar(_) => panic!(),
        }
    }
}

impl Domain<NoCost> {
    pub fn full() -> Self {
        Full
    }
    pub fn gap_start() -> Self {
        GapStart
    }
    pub fn gap_gap() -> Self {
        GapGap
    }
    pub fn dijkstra() -> Self {
        Astar(NoCost)
    }
}

impl Domain<GapCost> {
    pub fn dist_gap() -> Self {
        Astar(GapCost)
    }
}

impl<H> Domain<H> {
    pub fn astar(h: H) -> Self {
        Astar(h)
    }

    pub fn h(&self) -> Option<&H> {
        match self {
            Astar(h) => Some(&h),
            _ => None,
        }
    }
    pub fn h_mut(&mut self) -> Option<&mut H> {
        match self {
            Astar(h) => Some(h),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum DoublingStart {
    Zero,
    Gap,
    H0,
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum Strategy {
    None,
    BandDoubling { start: DoublingStart, factor: f32 },
    LinearSearch { start: DoublingStart, delta: f32 },
    LocalDoubling,
}
impl Strategy {
    pub fn band_doubling() -> Strategy {
        Self::BandDoubling {
            start: DoublingStart::H0,
            factor: 2.0,
        }
    }
}
impl Default for Strategy {
    fn default() -> Self {
        Strategy::BandDoubling {
            start: DoublingStart::H0,
            factor: 2.,
        }
    }
}
