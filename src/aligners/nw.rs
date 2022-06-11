use super::{Aligner, NoVisualizer, Visualizer};
use crate::cost_model::*;
use crate::prelude::{Pos, Sequence, I};
use std::cmp::min;
use std::mem::swap;

pub struct NW<CM: CostModel> {
    pub cm: CM,
}

// TODO: Instead use saturating add everywhere?
const INF: Cost = Cost::MAX / 2;

impl NW<LinearCost> {
    /// Computes the next layer from the current one.
    /// `ca` is the `i`th character of sequence `a`.
    #[inline]
    fn next_layer(
        &self,
        i: usize,
        ca: u8,
        b: &Sequence,
        prev: &Vec<Cost>,
        next: &mut Vec<Cost>,
        v: &mut impl Visualizer,
    ) {
        v.expand(Pos(i as I + 1, 0));
        next[0] = (i + 1) as Cost * self.cm.ins();
        for (j, &cb) in b.iter().enumerate() {
            v.expand(Pos(i as I + 1, j as I + 1));
            next[j + 1] = min(
                // Convert sub_cost to INF when substitutions are not allowed.
                prev[j].saturating_add(self.cm.sub_cost(ca, cb).unwrap_or(INF)),
                min(next[j] + self.cm.ins(), prev[j + 1] + self.cm.del()),
            );
        }
    }
}

impl Aligner for NW<LinearCost> {
    type Params = ();

    /// The cost-only version uses linear memory.
    fn cost(&self, a: &Sequence, b: &Sequence, _params: Self::Params) -> Cost {
        let mut prev = vec![INF; b.len() + 1];
        let mut next = vec![INF; b.len() + 1];
        next[0] = 0;
        for j in 1..=b.len() {
            next[j] = j as Cost * self.cm.del();
        }
        // TODO: Does enumerate_from exist?
        for (i, &ca) in a.iter().enumerate() {
            swap(&mut next, &mut prev);
            self.next_layer(i, ca, b, &prev, &mut next, &mut NoVisualizer);
        }

        return next[a.len()];
    }

    // NOTE: NW does not explore states; it only expands them.
    fn visualize(
        &self,
        a: &Sequence,
        b: &Sequence,
        _params: Self::Params,
        visualizer: &mut impl Visualizer,
    ) -> Cost {
        let mut m = vec![vec![INF; b.len() + 1]; a.len() + 1];
        m[0][0] = 0;
        visualizer.expand(Pos(0, 0));
        for j in 1..=b.len() {
            visualizer.expand(Pos(0, j as I));
            m[0][j] = j as Cost * self.cm.ins();
        }
        for (i, &ca) in a.iter().enumerate() {
            // We can't pass m[i] and m[i+1] both at the same time, so we must split the vector instead.
            // TODO: Is there a `get_two` method somewhere?
            let [ref mut prev, ref mut next] = m[i..i + 2];
            self.next_layer(i, ca, b, prev, next, visualizer);
        }

        return m[a.len()][b.len()];
    }
}
