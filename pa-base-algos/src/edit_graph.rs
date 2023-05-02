//! `EditGraph` is an abstraction for the graph corresponding to a cost model.
//! There may be multiple graphs corresponding to the same cost model:
//! https://research.curiouscoding.nl/posts/diagonal-transition-variations/

use pa_affine_types::*;
use pa_types::*;
use std::cmp::max;

use super::dt::Fr;

pub type AffineCigarOps = [Option<AffineCigarOp>; 2];

pub trait StateT: std::fmt::Debug {
    fn is_root(&self) -> bool;
    fn pos(&self) -> Pos;
}

/// NOTE: These functions assume padding from NW.
impl StateT for State {
    #[inline]
    fn is_root(&self) -> bool {
        self.i == 1 && self.j == 1 && self.layer.is_none()
    }

    #[inline]
    fn pos(&self) -> Pos {
        Pos::from(self.i, self.j)
    }
}

/// For now, `EditGraph` is simply a type containing functions to iterate over
/// the parents of a given position.
///
/// NOTE: It is important that all edges between different affine layers have positive (>0) cost!
/// This ensures that the order of iteration over the layers is not important.
/// The only exception is a gap-close cost of 0, when no backward iteration is done.
/// This is fine, because the main layer is iterated last.
///
/// The transitions to the affine layers are as follows:
/// gap-open:   cost o fr 1
/// gap-extend: cost e fr 1
/// gap-close:  cost e fr 0
///
/// NOTE: An alternative, with a lower maximum edge cost, would be:
/// gap-open:   cost floor((o+e)/2) fr 0
/// gap-extend: cost e              fr 1
/// gap-close:  cost ceil( (o+e)/2) fr 1
///
/// NOTE that this is different from the default WFA formulation, which uses
/// gap-open:   cost o+e fr 1
/// gap-extend: cost e   fr 1
/// gap-close:  cost 0   fr 0
///
/// TODO: Generalize to CostModel trait, instead of only AffineCost.
/// TODO: Make EditGraph a trait instead of type? Then we can implement
/// different graph shapes in different types, and each can be optimized individually.
/// TODO: Decide whether this is a type only class that takes all arguments on
/// each invocation, or whether it owns the sequences and cost model, and all
/// inspection of them has to go through it.
pub struct EditGraph;

impl EditGraph {
    pub fn max_edge_cost<const N: usize>(cm: &AffineCost<N>) -> Cost {
        let mut e = 0;
        e = max(e, cm.sub.unwrap_or_default());
        e = max(e, cm.ins.unwrap_or_default());
        e = max(e, cm.del.unwrap_or_default());
        for cml in &cm.affine {
            e = max(e, cml.open);
            e = max(e, cml.extend);
        }
        e
    }

    /// Iterate over the states/layers at the given position in 'the right'
    /// order, making sure dependencies within the states at the given position
    /// come first.
    ///
    /// I.e., in the normal case affine layers are iterated before the main
    /// layer, to ensure that the ends of the gap-close edges within this
    /// position are visited first.
    pub fn iterate_layers<const N: usize>(_cm: &AffineCost<N>, mut f: impl FnMut(Layer)) {
        for layer in 0..N {
            f(Some(layer));
        }
        f(None);
    }

    /// Iterate over the parents of the given state by calling `f` for each of them.
    /// Parents of a state are closer to (0,0) that the state itself.
    ///
    /// `iterate_children` may be needed as well at some point, but currently we
    /// use a 'pull-based' DP, meaning that in each state we look back, and
    /// never 'push' to the children.
    pub fn iterate_parents<const N: usize>(
        a: Seq,
        b: Seq,
        cm: &AffineCost<N>,
        greedy_matching: bool,
        State { i, j, layer }: State,
        mut f: impl FnMut(I, I, Layer, Cost, AffineCigarOps),
    ) {
        match layer {
            None => {
                // In the main layer, there are many possible edges:
                // - match
                // -  substitution
                // - insertion
                // - deletion
                // - affine close (insertion or deletion)

                // match / substitution
                let is_match = i > 0 && j > 0 && a[i as usize - 1] == b[j as usize - 1];
                if is_match {
                    f(-1, -1, None, 0, [Some(AffineCigarOp::Match), None]);
                    if greedy_matching {
                        return;
                    }
                } else if let Some(cost) = cm.sub {
                    f(-1, -1, None, cost, [Some(AffineCigarOp::Sub), None]);
                }

                // insertion
                if let Some(cost) = cm.ins {
                    f(0, -1, None, cost, [Some(AffineCigarOp::Ins), None]);
                }

                // deletion
                if let Some(cost) = cm.del {
                    f(-1, 0, None, cost, [Some(AffineCigarOp::Del), None]);
                }

                // affine close
                for (layer, cml) in cm.affine.iter().enumerate() {
                    f(
                        0,
                        0,
                        Some(layer),
                        cml.extend,
                        [Some(AffineCigarOp::AffineClose(layer)), None],
                    );
                }
            }
            Some(layer) => {
                // If we are currently in an affine layer, there are two options:
                // - gap open of cost `open + extend`, which is always allowed.
                // - gap close of cost `extend`, which in case of homopolymer
                //   layers is only allowed when the character to be extended
                //   equals the previous character.

                // gap open
                let cml = &cm.affine[layer];
                let (di, dj, op) = match cml.affine_type {
                    AffineLayerType::InsertLayer => (0, -1, AffineCigarOp::AffineIns(layer)),
                    AffineLayerType::DeleteLayer => (-1, 0, AffineCigarOp::AffineDel(layer)),
                };
                f(
                    di,
                    dj,
                    None,
                    cml.open,
                    [Some(op), Some(AffineCigarOp::AffineOpen(layer))],
                );

                // gap extend
                f(di, dj, Some(layer), cml.extend, [Some(op), None]);
            }
        }
    }

    /// Iterates parents for DT algorithms.
    /// Here, the exact position of the edge is not known until after the potential edge has been looked up.
    /// Thus, we split the callback into two parts:
    /// - f: gets the fr. point for the start of the given edge.
    /// - g: handles the edge, if it is indeed allowed.
    ///
    /// NOTE: Matches are completely ignored here.
    // FIXME: Cleanup redundant arguments now that HomoPolymer is removed.
    pub fn iterate_parents_dt<const N: usize>(
        cm: &AffineCost<N>,
        layer: Layer,
        // Given (di, dj) return the (i, j) of the end of the actual edge.
        mut f: impl FnMut(Fr, Fr, Layer, Cost) -> Option<(Fr, Fr)>,
        // Given `fr`, update fr point.
        mut g: impl FnMut(Fr, Fr, Fr, Fr, Layer, Cost, AffineCigarOps),
    ) {
        match layer {
            None => {
                // In the main layer, there are many possible edges:
                // - ~match~
                // -  substitution
                // - insertion
                // - deletion
                // - affine close (insertion or deletion)

                if let Some(cost) = cm.sub {
                    if let Some((i, j)) = f(-1, -1, None, cost) {
                        g(-1, -1, i, j, None, cost, [Some(AffineCigarOp::Sub), None]);
                    }
                }

                // insertion
                if let Some(cost) = cm.ins {
                    if let Some((i, j)) = f(0, -1, None, cost) {
                        g(0, -1, i, j, None, cost, [Some(AffineCigarOp::Ins), None]);
                    }
                }

                // deletion
                if let Some(cost) = cm.del {
                    if let Some((i, j)) = f(-1, 0, None, cost) {
                        g(-1, 0, i, j, None, cost, [Some(AffineCigarOp::Del), None]);
                    }
                }

                // affine close
                for (layer, cml) in cm.affine.iter().enumerate() {
                    if let Some((i, j)) = f(0, 0, Some(layer), cml.extend) {
                        g(
                            0,
                            0,
                            i,
                            j,
                            Some(layer),
                            cml.extend,
                            [Some(AffineCigarOp::AffineClose(layer)), None],
                        );
                    }
                }
            }
            Some(layer) => {
                // If we are currently in an affine layer, there are two options:
                // - gap open of cost `open + extend`, which is always allowed.
                // - gap close of cost `extend`, which in case of homopolymer
                //   layers is only allowed when the character to be extended
                //   equals the previous character.

                // gap open
                let cml = &cm.affine[layer];
                let (di, dj, op) = match cml.affine_type {
                    AffineLayerType::InsertLayer => (0, -1, AffineCigarOp::AffineIns(layer)),
                    AffineLayerType::DeleteLayer => (-1, 0, AffineCigarOp::AffineDel(layer)),
                };
                if let Some((i, j)) = f(di, dj, None, cml.open) {
                    g(
                        di,
                        dj,
                        i,
                        j,
                        None,
                        cml.open,
                        [Some(op), Some(AffineCigarOp::AffineOpen(layer))],
                    );
                }

                // gap extend
                if let Some((i, j)) = f(di, dj, Some(layer), cml.extend) {
                    g(di, dj, i, j, Some(layer), cml.extend, [Some(op), None]);
                }
            }
        }
    }

    /// Same as iterate_parent, but in the other direction.
    pub fn iterate_children_dt<const N: usize>(
        cm: &AffineCost<N>,
        layer: Layer,
        // Given (di, dj) return the (i, j) of the end of the actual edge.
        mut f: impl FnMut(Fr, Fr, Layer, Cost) -> Option<(Fr, Fr)>,
        // Given `fr`, update fr point.
        mut g: impl FnMut(Fr, Fr, Fr, Fr, Layer, Cost, AffineCigarOps),
    ) {
        match layer {
            None => {
                // In the main layer, there are many possible edges:
                // - ~match~
                // -  substitution
                // - insertion
                // - deletion
                // - affine close (insertion or deletion)

                if let Some(cost) = cm.sub {
                    if let Some((i, j)) = f(1, 1, None, cost) {
                        g(1, 1, i, j, None, cost, [Some(AffineCigarOp::Sub), None]);
                    }
                }

                // insertion
                if let Some(cost) = cm.ins {
                    if let Some((i, j)) = f(0, 1, None, cost) {
                        g(0, 1, i, j, None, cost, [Some(AffineCigarOp::Ins), None]);
                    }
                }

                // deletion
                if let Some(cost) = cm.del {
                    if let Some((i, j)) = f(1, 0, None, cost) {
                        g(1, 0, i, j, None, cost, [Some(AffineCigarOp::Del), None]);
                    }
                }

                // affine open
                for (layer, cml) in cm.affine.iter().enumerate() {
                    let (di, dj, op) = match cml.affine_type {
                        AffineLayerType::InsertLayer => (0, 1, AffineCigarOp::AffineIns(layer)),
                        AffineLayerType::DeleteLayer => (1, 0, AffineCigarOp::AffineDel(layer)),
                    };
                    if let Some((i, j)) = f(di, dj, Some(layer), cml.open) {
                        g(
                            di,
                            dj,
                            i,
                            j,
                            Some(layer),
                            cml.open,
                            [Some(AffineCigarOp::AffineOpen(layer)), Some(op)],
                        );
                    }
                }
            }
            Some(layer) => {
                // TODO
                // If we are currently in an affine layer, there are two options:
                // - gap close of cost `0`, which is always allowed.
                // - gap close of cost `extend`, which in case of homopolymer
                //   layers is only allowed when the character to be extended
                //   equals the previous character.

                // gap extend
                let cml = &cm.affine[layer];
                let (di, dj, op) = match cml.affine_type {
                    AffineLayerType::InsertLayer => (0, 1, AffineCigarOp::AffineIns(layer)),
                    AffineLayerType::DeleteLayer => (1, 0, AffineCigarOp::AffineDel(layer)),
                };
                if let Some((i, j)) = f(di, dj, Some(layer), cml.extend) {
                    g(di, dj, i, j, Some(layer), cml.extend, [Some(op), None]);
                }

                // affine close
                // NOTE: gap-close edges have a cost of 0 and stay in the current position.
                // This requires that the iteration order over layers at the current position visits the main layer first.
                if let Some((i, j)) = f(0, 0, None, cml.extend) {
                    g(
                        0,
                        0,
                        i,
                        j,
                        None,
                        cml.extend,
                        [Some(AffineCigarOp::AffineClose(layer)), None],
                    );
                }
            }
        }
    }
}
