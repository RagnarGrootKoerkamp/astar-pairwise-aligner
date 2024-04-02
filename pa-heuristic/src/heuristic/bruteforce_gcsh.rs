use itertools::Itertools;
use rand::Rng;
use std::cmp::Reverse;

use super::*;
use crate::{prune::MatchPruner, *};

#[derive(Debug, Copy, Clone)]
pub struct BruteForceGCSH<DH: Distance> {
    pub match_config: MatchConfig,
    pub distance_function: DH,
    pub pruning: Pruning,
}

impl<DH: Distance> Heuristic for BruteForceGCSH<DH>
where
    for<'a> DH::DistanceInstance<'a>: HeuristicInstance<'a>,
{
    type Instance<'a> = BruteForceGCSHI<'a, DH>;

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>) -> Self::Instance<'a> {
        BruteForceGCSHI::new(a, b, *self)
    }

    fn name(&self) -> String {
        "Seed".into()
    }
}

pub struct BruteForceGCSHI<'a, DH: Distance> {
    params: BruteForceGCSH<DH>,
    distance_function: DH::DistanceInstance<'a>,
    target: Pos,

    seeds: Seeds,
    matches: MatchPruner,

    // The lowest cost match starting at each position.
    h_at_matches: HashMap<Pos, Cost>,

    stats: HeuristicStats,
}

/// The seed heuristic implies a distance function as the maximum of the
/// provided distance function and the potential difference between the two
/// positions.  Assumes that the current position is not a match, and no matches
/// are visited in between `from` and `to`.
impl<'a, DH: Distance> DistanceInstance<'a> for BruteForceGCSHI<'a, DH>
where
    DH::DistanceInstance<'a>: DistanceInstance<'a>,
{
    fn distance(&self, from: Pos, to: Pos) -> Cost {
        max(
            self.distance_function.distance(from, to),
            self.seeds.potential_distance(from, to),
        )
    }
}

impl<'a, DH: Distance> BruteForceGCSHI<'a, DH>
where
    DH::DistanceInstance<'a>: DistanceInstance<'a>,
{
    fn new(a: Seq<'a>, b: Seq<'a>, params: BruteForceGCSH<DH>) -> Self {
        let Matches { seeds, mut matches } = find_matches(a, b, params.match_config, false);
        matches.sort_by_key(|m| LexPos(m.start));
        let num_matches = matches.len();
        let num_filtered_matches = matches.len();

        let mut h = BruteForceGCSHI::<'a> {
            params,
            distance_function: Distance::build(&params.distance_function, a, b),
            target: Pos::target(a, b),
            matches: MatchPruner::new(
                params.pruning,
                // Ensure consistency with GCSH.
                params.distance_function.name() == "Gap",
                matches,
                &seeds,
            ),
            seeds,
            h_at_matches: Default::default(),
            stats: Default::default(),
        };

        h.build();

        h.stats.h0 = h.h(Pos(0, 0));
        h.stats.num_seeds = h.seeds.seeds.len() as _;
        h.stats.num_matches = num_matches;
        h.stats.num_filtered_matches = num_filtered_matches;
        h
    }

    // A separate function that can be reused with pruning.
    fn build(&mut self) {
        self.h_at_matches.clear();
        self.h_at_matches.insert(self.target, 0);
        let mut matches = self.matches.iter().filter(|m| m.is_active()).collect_vec();
        matches.sort_by_key(|m| LexPos(m.start));

        for Match {
            start,
            end,
            match_cost,
            ..
        } in matches.iter().rev()
        {
            // Use the match.
            let update_val = *match_cost as Cost + self.h(*end);
            // Skip the match.
            let query_val = self.h(*start);

            // Update if using is better than skipping.
            // TODO: Report some metrics on skipped states.
            if update_val < query_val {
                self.h_at_matches.insert(*start, update_val);
            }
        }
    }
}

impl<'a, DH: Distance> HeuristicInstance<'a> for BruteForceGCSHI<'a, DH>
where
    DH::DistanceInstance<'a>: DistanceInstance<'a>,
{
    fn h(&self, pos: Pos) -> Cost {
        self.h_at_matches
            .iter()
            .filter(|&(parent, _)| pos <= *parent)
            .map(|(parent, val)| self.distance(pos, *parent).saturating_add(*val))
            .min()
            .unwrap()
    }

    fn h_with_parent(&self, pos: Pos) -> (Cost, Pos) {
        let rng = &mut rand::thread_rng();
        self.h_at_matches
            .iter()
            .filter(|&(parent, _)| *parent >= pos)
            .map(|(parent, val)| (self.distance(pos, *parent).saturating_add(*val), *parent))
            .min_by_key(|(val, pos)| (*val, 0 * rng.gen_range(0..u64::MAX), Reverse(LexPos(*pos))))
            .unwrap()
    }

    fn layer(&self, pos: Pos) -> Option<Cost> {
        let naive_dist = max(self.seeds.potential(pos), self.distance(pos, self.target));
        let h = self.h(pos);
        assert!(h <= naive_dist);
        Some(naive_dist - h)
    }

    /// TODO: This is copied from CSH::prune. It would be better to have a single implementation for this.
    fn prune(&mut self, pos: Pos, _hint: Self::Hint) -> (Cost, ()) {
        if !self.params.pruning.is_enabled() {
            return (0, ());
        }

        let (p_start, p_end) = self.matches.prune(&self.seeds, pos, |_m| {});

        if p_start + p_end > 0 {
            self.stats.num_pruned += p_start + p_end;
            self.build();
        }

        (0, ())
    }

    fn stats(&mut self) -> HeuristicStats {
        self.stats.h0_end = self.h(Pos(0, 0));
        self.stats
    }

    fn matches(&self) -> Option<Vec<Match>> {
        Some(self.matches.iter().cloned().collect_vec())
    }

    fn seeds(&self) -> Option<&Seeds> {
        Some(&self.seeds)
    }

    fn params_string(&self) -> String {
        format!("{:?}", self.params)
    }
}
