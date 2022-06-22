use super::cigar::{Cigar, CigarOp};
use super::front::Layers;
use super::NoVisualizer;
use super::{Aligner, VisualizerT};
use crate::cost_model::*;
use crate::prelude::{Pos, Sequence, I};
use std::cmp::{max, min};

pub type PATH = Vec<(usize, usize)>;
pub struct NW<CostModel> {
    pub cm: CostModel,
}

// TODO: Instead use saturating add everywhere?
const INF: Cost = Cost::MAX / 2;

/// The base vector M, and one vector per affine layer.
/// TODO: Possibly switch to a Vec<Layer> instead.
type Front<const N: usize> = super::front::Front<N, Cost, usize>;

impl<const N: usize> NW<AffineCost<N>> {
    pub(super) fn track_path(
        &self,
        fronts: &mut Vec<Front<N>>,
        a: &Sequence,
        b: &Sequence,
    ) -> (PATH, Cigar) {
        let mut path: PATH = vec![];
        let mut cigar = Cigar::default();

        // The current position and affine layer.
        let mut i = a.len();
        let mut j = b.len();
        // None for main layer.
        let mut layer: Option<usize> = None;

        path.push((i, j));

        let mut save = |x: usize, y: usize, op: CigarOp| {
            println!("save {x} {y} {op:?}");
            cigar.push(op);
            if let Some(last) = path.last() {
                if *last == (x, y) {
                    return;
                }
            }
            path.push((x, y));
        };
        'path_loop: while i > 0 || j > 0 {
            if let Some(layer_idx) = layer {
                match self.cm.affine[layer_idx].affine_type {
                    InsertLayer => {
                        if fronts[i].affine(layer_idx)[j]
                            == fronts[i].affine(layer_idx)[j - 1] + self.cm.affine[layer_idx].extend
                        {
                            // deletion gap extention from current affine layer
                            j -= 1;
                            save(i, j, CigarOp::AffineInsertion(layer_idx));
                            continue 'path_loop;
                        } else {
                            assert_eq!(
                                fronts[i].affine(layer_idx)[j], fronts[i].m()[j-1]
                                        + self.cm.affine[layer_idx].open
                                        + self.cm.affine[layer_idx].extend,"Path tracking error! No trace from deletion layer number {layer_idx}, coordinates {i}, {j}"
                            );
                            // Open new deletion gap from main layer
                            j -= 1;
                            save(i, j, CigarOp::AffineInsertion(layer_idx));
                            save(i, j, CigarOp::AffineOpen(layer_idx));
                            layer = None;
                            continue 'path_loop;
                        }
                    }
                    DeleteLayer => {
                        if fronts[i].affine(layer_idx)[j]
                            == fronts[i - 1].affine(layer_idx)[j] + self.cm.affine[layer_idx].extend
                        {
                            // insertion gap extention from current affine layer
                            i -= 1;
                            save(i, j, CigarOp::AffineDeletion(layer_idx));
                            continue 'path_loop;
                        } else {
                            assert_eq!(
                                fronts[i].affine(layer_idx)[j], fronts[i - 1].m()[j]
                                        + self.cm.affine[layer_idx].open
                                        + self.cm.affine[layer_idx].extend,"Path tracking error! No trace from insertion layer number {layer_idx}, coordinates {i}, {j}"
                            );
                            // opening new insertion gap from main layer
                            i -= 1;
                            save(i, j, CigarOp::AffineDeletion(layer_idx));
                            save(i, j, CigarOp::AffineOpen(layer_idx));
                            layer = None;
                            continue 'path_loop;
                        }
                    }
                    _ => todo!(),
                };
            } else {
                if i > 0 && j > 0 {
                    // match?
                    if a[i - 1] == b[j - 1] && fronts[i].m()[j] == fronts[i - 1].m()[j - 1] {
                        i -= 1;
                        j -= 1;
                        save(i, j, CigarOp::Match);
                        continue 'path_loop;
                    }
                    // mismatch?
                    if let Some(sub) = self.cm.sub {
                        if fronts[i].m()[j] == fronts[i - 1].m()[j - 1] + sub {
                            i -= 1;
                            j -= 1;
                            save(i, j, CigarOp::Mismatch);
                            continue 'path_loop;
                        }
                    }
                }
                // insertion?
                if j > 0 {
                    if let Some(ins) = self.cm.ins {
                        println!(
                            "ins={ins}  now={}  prev={}",
                            fronts[i].m()[j],
                            fronts[i].m()[j - 1]
                        );
                        if fronts[i].m()[j] == fronts[i].m()[j - 1] + ins {
                            j -= 1;
                            save(i, j, CigarOp::Insertion);
                            continue 'path_loop;
                        }
                    }
                }
                // deletion?
                if i > 0 {
                    if let Some(del) = self.cm.del {
                        println!(
                            "del={del}  now={}  prev={}",
                            fronts[i].m()[j],
                            fronts[i - 1].m()[j]
                        );
                        if fronts[i].m()[j] == fronts[i - 1].m()[j] + del {
                            i -= 1;
                            save(i, j, CigarOp::Deletion);
                            continue 'path_loop;
                        }
                    }
                }
                // Affine layers check
                // NOTE: This loop does not change the position, only the layer.
                for parent_layer_idx in 0..N {
                    if fronts[i].m()[j] == fronts[i].affine(parent_layer_idx)[j] {
                        layer = Some(parent_layer_idx);
                        save(i, j, CigarOp::AffineClose(parent_layer_idx));
                        continue 'path_loop;
                    }
                }
            }
            panic!("Did not find parent on path!\nIn ({i}, {j}) at layer {layer:?} with cost ");
        }
        path.reverse();
        cigar.reverse();
        (path, cigar)
    }

    /// Computes the next front (front `i`) from the current one.
    /// `ca` is the `i-1`th character of sequence `a`.
    pub(super) fn next_front(
        &self,
        i: usize,
        ca: u8,
        b: &Sequence,
        prev: &Front<N>,
        next: &mut Front<N>,
        v: &mut impl VisualizerT,
    ) {
        v.expand(Pos(i as I, 0));
        // TODO: Instead of manually doing the first state, it is also possible
        // to simply add a buffer layer around the DP. The issue with that
        // however, is that we would need to prefix both sequences with the same
        // unique character to have a place to look at.
        // NOTE: This MUST be fixed for this to work with partial fronts from exponential search.

        if next.range.contains(&0) {
            // Initialize the first state by linear deletion.
            next.m_mut()[0] = self.cm.del_or(INF, |del| i as Cost * del);
            // Initialize the first state by affine deletion.
            for (layer_idx, cm) in self.cm.affine.iter().enumerate() {
                let (mut m, mut a) = next.m_affine_mut(layer_idx);
                match cm.affine_type {
                    InsertLayer => {}
                    DeleteLayer => {
                        a[0] = cm.open + i as Cost * cm.extend;
                        m[0] = min(m[0], a[0]);
                    }
                    _ => todo!(),
                };
            }
        }
        for j in max(*next.range.start(), 1)..=*next.range.end() {
            println!("j {j} len b {}", b.len());
            let cb = b[j - 1];

            // Compute all layers at (i, j).
            v.expand(Pos(i as I, j as I));

            // Main layer: substitutions and linear indels.
            let mut f = INF;
            // NOTE: When sub/ins/del is not allowed, we have to skip them.
            // TODO: When a match is possible, we could skip all other options.
            if ca == cb {
                f = min(f, prev.m()[j - 1]);
            } else {
                if let Some(sub) = self.cm.sub {
                    f = min(f, prev.m()[j - 1] + sub);
                }
            }
            if let Some(ins) = self.cm.ins {
                f = min(f, next.m()[j - 1] + ins);
            }
            if let Some(del) = self.cm.del {
                f = min(f, prev.m()[j] + del);
            }

            // Affine layers
            // TODO: Swap the order of this for loop and the loop over j?
            for (layer_idx, cm) in self.cm.affine.iter().enumerate() {
                let (next_m, mut next_affine_layer) = next.m_affine_mut(layer_idx);
                match cm.affine_type {
                    InsertLayer => {
                        next_affine_layer[j] = min(
                            next_affine_layer[j - 1] + cm.extend,
                            next_m[j - 1] + cm.open + cm.extend,
                        )
                    }
                    DeleteLayer => {
                        next_affine_layer[j] = min(
                            prev.affine(layer_idx)[j] + cm.extend,
                            prev.m()[j] + cm.open + cm.extend,
                        )
                    }
                    _ => todo!(),
                };
                f = min(f, next_affine_layer[j]);
            }

            next.m_mut()[j] = f;
        }
    }

    fn init_front(&self, b: &Sequence) -> Front<N> {
        let mut next = Front {
            layers: Layers::new(vec![INF; b.len() + 1]),
            // unused
            range: 0..=b.len(),
            offset: 0,
        };

        // TODO: Find a way to not have to manually process the first layer.
        next.m_mut()[0] = 0;
        for j in 1..=b.len() {
            // Initialize the main layer with linear insertions.
            next.m_mut()[j] = self.cm.ins_or(INF, |ins| j as Cost * ins);

            // Initialize the affine insertion layers.
            for (layer_idx, cm) in self.cm.affine.iter().enumerate() {
                let (mut next_m, mut next_layer) = next.m_affine_mut(layer_idx);
                match cm.affine_type {
                    InsertLayer => {
                        next_layer[j] = cm.open + j as Cost * cm.extend;
                    }
                    DeleteLayer => {}
                    _ => todo!(),
                };
                next_m[j] = min(next_m[j], next_layer[j]);
            }
        }
        next
    }
}

impl<const N: usize> Aligner for NW<AffineCost<N>> {
    /// The cost-only version uses linear memory.
    fn cost(&self, a: &Sequence, b: &Sequence) -> Cost {
        let ref mut next = self.init_front(b);
        let ref mut prev = next.clone();

        for (i0, &ca) in a.iter().enumerate() {
            // Convert to 1 based index.
            let i = i0 + 1;
            std::mem::swap(prev, next);
            self.next_front(i, ca, b, prev, next, &mut NoVisualizer);
        }

        return next.m()[b.len()];
    }

    fn visualize(
        &self,
        a: &Sequence,
        b: &Sequence,
        v: &mut impl VisualizerT,
    ) -> (Cost, PATH, Cigar) {
        let ref mut fronts = vec![
            Front {
                layers: Layers::new(vec![INF; b.len() + 1]),
                // unused
                range: 0..=b.len(),
                offset: 0,
            };
            a.len() + 1
        ];
        // TODO: Reuse memory instead of overwriting it.
        fronts[0] = self.init_front(b);

        for j in 0..=b.len() {
            v.expand(Pos(0, j as I));
        }

        for (i0, &ca) in a.iter().enumerate() {
            // Change from 0-based to 1-based indexing.
            let i = i0 + 1;
            let [prev, next] = &mut fronts[i-1..=i] else {unreachable!();};
            self.next_front(i, ca, b, prev, next, v);
        }

        let d = fronts[a.len()].m()[b.len()];
        let (path, cigar) = self.track_path(fronts, a, b);
        return (d, path, cigar);
    }
}
