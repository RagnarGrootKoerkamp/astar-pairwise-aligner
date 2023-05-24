use super::*;
use crate::contour::sh_contours::{self, Arrow, ShContours};
use crate::prune::MatchPruner;
use crate::util::Timer;
use crate::*;

#[derive(Debug, Copy, Clone)]
pub struct SH {
    pub match_config: MatchConfig,
    pub pruning: Pruning,
}

impl SH {
    pub fn new(match_config: MatchConfig, pruning: Pruning) -> Self {
        Self {
            match_config,
            pruning,
        }
    }
}

impl Heuristic for SH {
    type Instance<'a> = SHI;

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>) -> Self::Instance<'a> {
        SHI::new(a, b, *self)
    }

    fn name(&self) -> String {
        "SH".into()
    }
}

pub struct SHI {
    params: SH,

    seeds: Seeds,
    matches: MatchPruner,
    contours: ShContours,

    /// The maximum position explored so far.
    max_explored_pos: Pos,

    stats: HeuristicStats,
}

impl SHI {
    fn new(a: Seq, b: Seq, params: SH) -> Self {
        let Matches { seeds, matches } = find_matches(a, b, params.match_config, false);

        let contours = ShContours::new(
            &seeds,
            matches.iter().map(match_to_arrow).rev(),
            params.match_config.r as I,
        );

        let mut stats = HeuristicStats::default();
        stats.num_seeds = seeds.seeds.len() as I;
        stats.num_matches = matches.len();
        stats.num_filtered_matches = matches.len();
        let mut h = SHI {
            params,
            max_explored_pos: Pos(0, 0),
            stats,
            matches: MatchPruner::new(params.pruning, false, matches, &seeds),
            seeds,
            contours,
        };
        h.stats.h0 = h.h(Pos(0, 0));
        h
    }
}

fn match_to_arrow(m: &Match) -> Arrow {
    Arrow {
        start: m.start.0,
        end: m.end.0,
        score: m.score(),
    }
}

type Hint = sh_contours::Hint;

impl<'a> HeuristicInstance<'a> for SHI {
    /// The index of the next match, from the end of the splitvec.
    type Hint = Hint;

    fn h(&self, pos: Pos) -> Cost {
        let p = self.seeds.potential(pos);
        let m = self.contours.score(pos.0);
        p - m
    }

    fn layer(&self, pos: Pos) -> Option<Cost> {
        Some(self.contours.score(pos.0))
    }

    fn layer_with_hint(&self, pos: Pos, hint: Self::Hint) -> Option<(Cost, Self::Hint)> {
        Some(self.contours.score_with_hint(pos.0, hint))
    }

    fn h_with_hint(&self, pos: Pos, hint: Self::Hint) -> (Cost, Self::Hint) {
        let p = self.seeds.potential(pos);
        let (m, h) = self.contours.score_with_hint(pos.0, hint);
        (p - m, h)
    }

    fn h_with_hint_timed(&mut self, pos: Pos, hint: Self::Hint) -> ((Cost, Self::Hint), f64) {
        let timer = Timer::new(&mut self.stats.h_calls);
        let ans = self.h_with_hint(pos, hint);
        let t = timer.end(&mut self.stats.h_duration);
        (ans, t)
    }

    fn root_potential(&self) -> Cost {
        self.seeds.potential[0]
    }

    type Order = I;

    fn prune(&mut self, pos: Pos, hint: Self::Hint) -> (Cost, I) {
        if !self.params.pruning.is_enabled() {
            return (0, 0);
        }

        // Time the duration of retrying once in this many iterations.
        let timer = Timer::new(&mut self.stats.prune_calls);

        let mut change = 0;
        let (p_start, p_end) = self.matches.prune(&self.seeds, pos, |m| {
            let c = self
                .contours
                .prune_with_hint(&self.seeds, match_to_arrow(m), hint);

            if m.start.0 == pos.0 {
                change += c;
            }
        });

        if p_start + p_end > 0 {
            self.stats.num_pruned += p_start + p_end;
        }

        timer.end(&mut self.stats.prune_duration);
        if pos >= self.max_explored_pos {
            (change, pos.0)
        } else {
            (0, 0)
        }
    }

    fn explore(&mut self, pos: Pos) {
        self.max_explored_pos.0 = max(self.max_explored_pos.0, pos.0);
        self.max_explored_pos.1 = max(self.max_explored_pos.1, pos.1);
    }

    fn stats(&mut self) -> HeuristicStats {
        self.stats.h0_end = self.h(Pos(0, 0));
        self.stats
    }

    fn matches(&self) -> Option<Vec<Match>> {
        Some(self.matches.iter().cloned().collect())
    }

    fn seeds(&self) -> Option<&Seeds> {
        Some(&self.seeds)
    }

    fn params_string(&self) -> String {
        format!("{:?}", self.params)
    }
}
