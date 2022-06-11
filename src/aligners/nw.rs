use super::Aligner;
use crate::prelude::*;
use std::mem::swap;

struct NW;

const INF: Cost = Cost::MAX;

impl Aligner for NW {
    type Params = ();
    type CostModel = LinearCost;

    /// The cost-only version uses linear memory.
    fn cost(cm: Self::CostModel, a: &Sequence, b: &Sequence, _params: Self::Params) -> Cost {
        let mut prev = vec![INF; b.len() + 1];
        let mut next = vec![INF; b.len() + 1];
        next[0] = 0;
        for j in 1..=b.len() {
            next[j] = j as Cost * cm.del();
        }
        // TODO: Does enumerate_from exist?
        for (i, &ca) in a.iter().enumerate() {
            swap(&mut next, &mut prev);
            next[0] = (i + 1) as Cost * cm.ins();
            for (j, &cb) in b.iter().enumerate() {
                next[j + 1] = min(
                    // Convert sub_cost to INF when substitutions are not allowed.
                    prev[j].saturating_add(cm.sub_cost(ca, cb).unwrap_or(INF)),
                    min(next[j] + cm.ins(), prev[j + 1] + cm.del()),
                );
            }
        }

        return next[a.len()];
    }

    // NOTE: NW does not explore states; it only expands them.
    fn visualize(
        cm: Self::CostModel,
        a: &Sequence,
        b: &Sequence,
        _params: Self::Params,
        visualizer: &mut impl aligners::Visualizer,
    ) -> Cost {
        let mut m = vec![vec![INF; b.len() + 1]; a.len() + 1];
        m[0][0] = 0;
        visualizer.expand(Pos(0, 0));
        for j in 1..=b.len() {
            visualizer.expand(Pos(0, j as I));
            m[0][j] = j as Cost * cm.ins();
        }
        for i in 1..=a.len() {
            visualizer.expand(Pos(i as I, 0));
            m[i][0] = i as Cost * cm.del();
        }
        for (i, &ca) in a.iter().enumerate() {
            let i = i + 1;
            for (j, &cb) in b.iter().enumerate() {
                let j = j + 1;
                visualizer.expand(Pos(i as I, j as I));
                m[i][j] = min(
                    // Convert sub_cost to INF when substitutions are not allowed.
                    m[i - 1][j - 1].saturating_add(cm.sub_cost(ca, cb).unwrap_or(INF)),
                    min(m[i][j - 1] + cm.ins(), m[i - 1][j] + cm.del()),
                );
            }
        }

        return m[a.len()][b.len()];
    }
}
