use super::Aligner;
use crate::prelude::*;
use std::mem::swap;

struct NW;

// TODO: Instead use saturating add everywhere?
const INF: Cost = Cost::MAX / 2;

impl NW {
    /// Computes the next layer from the current one.
    /// `ca` is the `i`th character of sequence `a`.
    fn next_layer(
        cm: &LinearCost,
        i: usize,
        ca: u8,
        b: &Sequence,
        prev: &Vec<Cost>,
        next: &mut Vec<Cost>,
    ) {
        next[0] = (i + 1) as Cost * cm.ins();
        for (j, &cb) in b.iter().enumerate() {
            next[j + 1] = min(
                // Convert sub_cost to INF when substitutions are not allowed.
                prev[j].saturating_add(cm.sub_cost(ca, cb).unwrap_or(INF)),
                min(next[j] + cm.ins(), prev[j + 1] + cm.del()),
            );
        }
    }
}

impl Aligner for NW {
    type Params = ();
    type CostModel = LinearCost;

    /// The cost-only version uses linear memory.
    fn cost(cm: &Self::CostModel, a: &Sequence, b: &Sequence, _params: Self::Params) -> Cost {
        let mut prev = vec![INF; b.len() + 1];
        let mut next = vec![INF; b.len() + 1];
        next[0] = 0;
        for j in 1..=b.len() {
            next[j] = j as Cost * cm.del();
        }
        // TODO: Does enumerate_from exist?
        for (i, &ca) in a.iter().enumerate() {
            swap(&mut next, &mut prev);
            NW::next_layer(cm, i, ca, b, &prev, &mut next);
        }

        return next[a.len()];
    }

    // NOTE: NW does not explore states; it only expands them.
    fn visualize(
        cm: &Self::CostModel,
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
        for (i, &ca) in a.iter().enumerate() {
            for j in 0..=b.len() {
                visualizer.expand(Pos(i as I + 1, j as I));
            }
            // We can't pass m[i] and m[i+1] both at the same time, so we must split the vector instead.
            // TODO: Is there a `get_two` method somewhere?
            let (front, back) = m.split_at_mut(i + 1);
            NW::next_layer(
                cm,
                i,
                ca,
                b,
                front.last().unwrap(),
                back.first_mut().unwrap(),
            );
        }

        return m[a.len()][b.len()];
    }
}
