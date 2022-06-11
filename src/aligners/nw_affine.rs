use super::{nw::NW, Aligner, Visualizer};
use crate::cost_model::*;
use crate::prelude::{Pos, Sequence, I};
use std::cmp::min;

// TODO: Instead use saturating add everywhere?
const INF: Cost = Cost::MAX / 2;

impl NW<AffineCost> {
    /// Computes the next layer from the current one.
    /// `ca` is the `i`th character of sequence `a`.
    fn next_layer_affine(
        &self,
        i: usize,
        ca: u8,
        b: &Sequence,
        ins: &mut [Vec<Cost>; 2],
        del: &mut [Vec<Cost>; 2],
        m: &mut [Vec<Cost>; 2],
    ) {
        del[1][0] = INF;
        ins[1][0] = self.cm.ins_open() + (i + 1) as Cost * self.cm.ins();
        m[1][0] = ins[(i + 1)][0];
        for (j, &cb) in b.iter().enumerate() {
            let j = j + 1;
            ins[1][j] = min(
                ins[0][j] + self.cm.ins(),
                m[0][j] + self.cm.ins_open() + self.cm.ins(),
            );
            del[1][j] = min(
                del[1][j - 1] + self.cm.del(),
                m[1][j - 1] + self.cm.del_open() + self.cm.del(),
            );
            m[1][j] = min(
                // Convert sub_cost to INF when substitutions are not allowed.
                m[0][j - 1].saturating_add(self.cm.sub_cost(ca, cb).unwrap_or(INF)),
                min(ins[1][j], del[1][j]),
            );
        }
    }
}

impl Aligner for NW<AffineCost> {
    /// The cost-only version uses linear memory.
    fn cost(&self, a: &Sequence, b: &Sequence, _params: Self::Params) -> Cost {
        // TODO: Make this a single 2D vec of structs instead?
        // NOTE: Index 0 and 1 correspond to `prev` and `next` in the non-affine `NW`.
        // End with an insertion.
        let mut ins = [vec![INF; b.len() + 1], vec![INF; b.len() + 1]];
        // End with a deletion.
        let mut del = [vec![INF; b.len() + 1], vec![INF; b.len() + 1]];
        // End with anything.
        let mut m = [vec![INF; b.len() + 1], vec![INF; b.len() + 1]];
        m[1][0] = 0;
        ins[1][0] = 0;
        del[1][0] = 0;
        for j in 1..=b.len() {
            del[1][j] = self.cm.del_open() + j as Cost * self.cm.del();
            m[1][j] = del[0][j];
        }
        for (i, &ca) in a.iter().enumerate() {
            ins.reverse();
            del.reverse();
            m.reverse();
            self.next_layer_affine(i, ca, b, &mut ins, &mut del, &mut m);
        }

        return m[1][b.len()];
    }

    fn visualize(
        &self,
        a: &Sequence,
        b: &Sequence,
        _params: Self::Params,
        visualizer: &mut impl Visualizer,
    ) -> Cost {
        // TODO: Make this a single 2D vec of structs instead?
        // End with an insertion.
        let mut ins = vec![vec![INF; b.len() + 1]; a.len() + 1];
        // End with a deletion.
        let mut del = vec![vec![INF; b.len() + 1]; a.len() + 1];
        // End with anything.
        let mut m = vec![vec![INF; b.len() + 1]; a.len() + 1];

        visualizer.expand(Pos(0, 0));
        m[0][0] = 0;
        ins[0][0] = 0;
        del[0][0] = 0;
        for j in 1..=b.len() {
            visualizer.expand(Pos(0, j as I));
            del[0][j] = self.cm.del_open() + j as Cost * self.cm.del();
            m[0][j] = del[0][j];
        }
        for (i, &ca) in a.iter().enumerate() {
            for j in 0..=b.len() {
                visualizer.expand(Pos(i as I + 1, j as I));
            }
            self.next_layer_affine(
                i,
                ca,
                b,
                // Get a mutable slice of 2 rows from each of the arrays.
                &mut ins[i..i + 2].as_chunks_mut::<2>().0[0],
                &mut del[i..i + 2].as_chunks_mut::<2>().0[0],
                &mut m[i..i + 2].as_chunks_mut::<2>().0[0],
            );
        }

        return m[a.len()][b.len()];
    }
}
