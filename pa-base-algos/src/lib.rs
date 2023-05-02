#![feature(let_chains, step_trait, int_roundings)]

use pa_types::Cost;
use serde::{Deserialize, Serialize};
use std::cmp::{max, min};

mod edit_graph;
mod front;

pub mod cli;
pub mod dt;
pub mod nw;

#[cfg(test)]
mod tests;

/// Find the cost using exponential search based on `f`.
///
/// Tries values `offset + s0 * f^i`.
fn exponential_search<T>(
    offset: Cost,
    s0: Cost,
    factor: f32,
    mut f: impl FnMut(Cost) -> Option<(Cost, T)>,
) -> (Cost, T) {
    let mut s = s0;
    let mut maxs = Cost::MAX;
    // TODO: Fix the potential infinite loop here.
    loop {
        if let Some((cost, t)) = f(offset + s) {
            if cost <= s {
                return (cost, t);
            } else {
                // If some value was returned this is an upper bound on the answer.
                maxs = min(maxs, cost);
            }
        }
        s = max((factor * s as f32).ceil() as Cost, 1);
        s = min(s, maxs);
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
    LocalDoubling,
}
impl Strategy {
    pub fn band_doubling() -> Strategy {
        Self::BandDoubling {
            start: DoublingStart::Gap,
            factor: 2.0,
        }
    }
}
