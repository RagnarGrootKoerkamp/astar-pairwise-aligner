use super::*;
use std::cmp::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub enum DoublingStart {
    Zero,
    Gap,
    H0,
}

impl DoublingStart {
    /// Return the start value and initial increment based on DoublingStart.
    pub fn initial_values(&self, a: &[u8], b: &[u8], h0: Cost) -> (i32, i32) {
        let (start_f, start_increment) = match self {
            DoublingStart::Zero => (0, 1),
            DoublingStart::Gap => {
                let x = pa_affine_types::AffineCost::unit().gap_cost(Pos(0, 0), Pos::target(a, b));
                (x, x)
            }
            DoublingStart::H0 => (h0, 1),
        };
        (start_f, start_increment)
    }
}

#[derive(Clone, Copy, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum DoublingType {
    None,
    BandDoubling {
        start: DoublingStart,
        factor: f32,
    },
    LinearSearch {
        start: DoublingStart,
        delta: f32,
    },
    LocalDoubling,
    /// For visualization purposes only.
    BandDoublingStartIncrement {
        start: DoublingStart,
        factor: f32,
        /// Start growing by this.
        start_increment: Cost,
    },
}

impl DoublingType {
    pub fn band_doubling() -> DoublingType {
        Self::BandDoubling {
            start: DoublingStart::Gap,
            factor: 2.0,
        }
    }
}

impl Default for DoublingType {
    fn default() -> Self {
        DoublingType::BandDoubling {
            start: DoublingStart::H0,
            factor: 2.,
        }
    }
}

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
pub fn exponential_search<T>(
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

pub fn linear_search<T>(
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
