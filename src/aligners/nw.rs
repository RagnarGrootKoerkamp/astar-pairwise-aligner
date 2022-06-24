use itertools::chain;

use super::cigar::{Cigar, CigarOp};
use super::Seq;
use super::{Aligner, VisualizerT};
use crate::cost_model::*;
use crate::prelude::{Pos, I};
use std::cmp::{max, min};
use std::ops::RangeInclusive;

pub type Path = Vec<Pos>;
pub struct NW<'a, CostModel, V: VisualizerT> {
    /// The cost model to use.
    pub cm: CostModel,

    /// When false, the band covers all states with distance <=s.
    /// When true, we only cover states with distance <=s/2.
    pub use_gap_cost_heuristic: bool,

    /// The visualizer to use.
    pub v: &'a mut V,
}

/// Type used for indexing sequences.
type Idx = isize;

// TODO: Instead use saturating add everywhere?
const INF: Cost = Cost::MAX / 2;

/// The base vector M, and one vector per affine layer.
/// TODO: Possibly switch to a Vec<Layer> instead.
type Front<const N: usize> = super::front::Front<N, Cost, Idx>;
type Fronts<const N: usize> = super::front::Fronts<N, Cost, Idx>;

/// NW DP only needs the cell just left and above of the current cell.
const LEFT_BUFFER: Idx = 1;
const RIGHT_BUFFER: Idx = 1;
/// Add one layer before the first, for easy initialization.
const TOP_BUFFER: Idx = 1;

impl<const N: usize, V: VisualizerT> NW<'_, AffineCost<N>, V> {
    fn track_path(&self, fronts: &Fronts<N>, a: Seq, b: Seq) -> (Path, Cigar) {
        let mut path: Path = vec![];
        let mut cigar = Cigar::default();

        // The current position and affine layer.
        let mut i = a.len() as Idx;
        let mut j = b.len() as Idx;
        // None for main layer.
        let mut layer: Option<usize> = None;

        path.push(Pos(i as I, j as I));

        let mut save = |x: Idx, y: Idx, op: CigarOp| {
            cigar.push(op);
            if let Some(last) = path.last() {
                if *last == Pos(x as I, y as I) {
                    return;
                }
            }
            path.push(Pos(x as I, y as I));
        };
        // TODO: Extract a parent() function and call that in a loop.
        'path_loop: while i > 0 || j > 0 || layer.is_some() {
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
                    if a[i as usize - 1] == b[j as usize - 1]
                        && fronts[i].m()[j] == fronts[i - 1].m()[j - 1]
                    {
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
    ///
    /// Call this with `i=0`, `ca='^'` and a `prev` front of appropriate size to fill the first layer,
    /// where `prev[-1] = 0`, so that the match between `^` and `^` makes `next[0] == 0` as well.
    fn next_front(&mut self, i: Idx, ca: u8, b: Seq, prev: &Front<N>, next: &mut Front<N>) {
        self.v.expand(Pos(i as I, 0));
        for j in next.range().clone() {
            // When j=0, we must use a placeholder. The actual character does not matter.
            // TODO: We could just try `b[-1]`?
            let cb = if j == 0 { b'^' } else { b[j as usize - 1] };
            println!("i {i} j {j} {ca} {cb}");

            // Compute all layers at (i, j).
            self.v.expand(Pos(i as I, j as I));

            // Main layer: substitutions and linear indels.
            let mut f = INF;
            // NOTE: When sub/ins/del is not allowed, we have to skip them.
            // TODO: When a match is possible, we could skip all other options.
            if ca == cb {
                f = min(f, prev.m()[j - 1]);
            } else {
                // TODO: This may be faster:
                // f = min(f, prev.m()[j-1] + sub.unwrap_or(INF))
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

    /// The first active row in column `i`, when searching up to distance `s`.
    fn j_range(&self, a: Seq, b: Seq, i: Idx, s: Option<Cost>) -> RangeInclusive<Idx> {
        let Some(s) = s else {
            return 0..=b.len() as Idx;
        };
        let i = i as isize;
        let s = s as isize;
        let range = if self.use_gap_cost_heuristic {
            let d = b.len() as isize - a.len() as isize;
            let per_band_cost = (self.cm.min_ins_extend + self.cm.min_del_extend) as isize;
            if d > 0 {
                let reduced_s = s - d * self.cm.min_ins_extend as isize;
                -(reduced_s / per_band_cost)..=d + reduced_s / per_band_cost
            } else {
                let reduced_s = s - d * self.cm.min_del_extend as isize;
                d - (reduced_s / per_band_cost)..=reduced_s / per_band_cost
            }
        } else {
            -(s / self.cm.min_del_extend as isize)..=(s / self.cm.min_ins_extend as isize)
        };
        // crop
        max(i + *range.start(), 0) as Idx..=min(i + *range.end(), b.len() as isize) as Idx
    }
}
impl<const N: usize, V: VisualizerT> Aligner for NW<'_, AffineCost<N>, V> {
    type CostModel = AffineCost<N>;

    fn cost_model(&self) -> &Self::CostModel {
        &self.cm
    }

    /// Test whether the cost is at most s.
    /// Returns None if cost > s, or the actual cost otherwise.
    fn cost_for_bounded_dist(&mut self, a: Seq, b: Seq, s_bound: Option<Cost>) -> Option<Cost> {
        println!("cost for bounded dist {s_bound:?}\n{a:?}\n{b:?}\n");
        let ref mut prev = Front::default();
        let ref mut next = Front::new(
            INF,
            self.j_range(a, b, 0, s_bound),
            LEFT_BUFFER,
            RIGHT_BUFFER,
        );
        // Initialize next[-1] = 0, so that the first layer will get next[0] = 0.
        *next.m_mut().negative_index(1) = 0;

        // NOTE: We compute the first front by passing `i=0` with character `'^'`.
        for (i, &ca) in chain(&[b'^'], a).enumerate() {
            let i = i as Idx;
            std::mem::swap(prev, next);
            // Update front size.
            next.reset(
                INF,
                self.j_range(a, b, i, s_bound),
                LEFT_BUFFER,
                RIGHT_BUFFER,
            );
            self.next_front(i, ca, b, prev, next);
            println!("{i} {ca}\n{next:?}");
        }

        if let Some(&dist) = next.m().get(b.len() as Idx) {
            println!("DIST: {dist:?}");
            Some(dist)
        } else {
            None
        }
    }

    /// Tries to find a path with cost <= s.
    /// Returns None if cost > s, or the actual cost otherwise.
    fn align_for_bounded_dist(
        &mut self,
        a: Seq,
        b: Seq,
        s_bound: Option<Cost>,
    ) -> Option<(Cost, Path, Cigar)> {
        let mut fronts = Fronts::new(
            INF,
            // The fronts to create.
            0..=a.len() as Idx,
            // The range for each front.
            |i| self.j_range(a, b, i, s_bound),
            TOP_BUFFER,
            0,
            LEFT_BUFFER,
            RIGHT_BUFFER,
        );
        // Initialize position (-1,-1) to 0, so that position (0,0) will be 0
        // after matching the first '^' character that we use when `i=0` and
        // `j=0`.
        *fronts[-1].m_mut().negative_index(1) = 0;

        for (i, &ca) in chain(&[b'^'], a).enumerate() {
            let i = i as Idx;
            let [prev, next] = &mut fronts[i-1..=i] else {unreachable!();};
            self.next_front(i, ca, b, prev, next);
        }

        if let Some(&dist) = fronts[a.len() as Idx].m().get(b.len() as Idx) {
            // We only track the actual path if `s` is small enough.
            if dist <= s_bound.unwrap_or(INF) {
                let (path, cigar) = self.track_path(&fronts, a, b);
                return Some((dist, path, cigar));
            }
        }
        None
    }
}
