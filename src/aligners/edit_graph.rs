//! `EditGraph` is an abstraction for the graph corresponding to a cost model.
//! There may be multiple graphs corresponding to the same cost model:
//! https://research.curiouscoding.nl/posts/diagonal-transition-variations/

use super::{cigar::CigarOp, Seq};
use crate::{
    cost_model::{AffineCost, AffineLayerType, Cost, CostModel},
    prelude::Pos,
};

pub type Layer = Option<usize>;
pub type I = isize;
pub type CigarOps = [Option<CigarOp>; 2];

#[derive(Clone, Copy, Debug)]
pub struct State {
    pub i: I,
    pub j: I,
    pub layer: Layer,
}

/// For now, `EditGraph` is simply a type containing functions to iterate over
/// the parents of a given position.
///
/// TODO: Generalize to CostModel trait, instead of only AffineCost.
/// TODO: Make EditGraph a trait instead of type? Then we can implement
/// different graph shapes in different types, and each can be optimized individually.
/// TODO: Decide whether this is a type only class that takes all arguments on
/// each invocation, or whether it owns the sequences and cost model, and all
/// inspection of them has to go through it.
pub struct EditGraph;

impl State {
    #[inline]
    pub fn new(i: I, j: I, layer: Layer) -> Self {
        Self { i, j, layer }
    }

    pub fn target(a: Seq, b: Seq) -> Self {
        Self {
            i: a.len() as I,
            j: b.len() as I,
            layer: None,
        }
    }

    pub fn pos(&self) -> Pos {
        Pos::from(self.i, self.j)
    }
}

impl EditGraph {
    /// Iterate over the parents of the given state by calling `f` for each of them.
    /// Parents of a state are closer to (0,0) that the state itself.
    ///
    /// `iterate_children` may be needed as well at some point, but currently we
    /// use a 'pull-based' DP, meaning that in each state we look back, and
    /// never 'push' to the children.
    #[inline]
    pub fn iterate_parents<const N: usize>(
        a: Seq,
        b: Seq,
        cm: &AffineCost<N>,
        greedy_matching: bool,
        State { i, j, layer }: State,
        f: impl FnMut(Layer, I, I, Cost, CigarOps),
    ) {
        match layer {
            None => {
                // In the main layer, there are many possible edges:
                // - match
                // - mismatch / substitution
                // - insertion
                // - deletion
                // - affine close (insertion or deletion)

                // match / mismatch
                let is_match = a[i as usize - 1] == b[j as usize - 1];
                if is_match {
                    f(None, -1, -1, 0, [Some(CigarOp::Match), None]);
                    if greedy_matching {
                        return;
                    }
                } else {
                    f(
                        None,
                        -1,
                        -1,
                        cm.sub.unwrap_or(Cost::MAX / 2),
                        [Some(CigarOp::Mismatch), None],
                    );
                }

                // insertion
                if j > 0 && let Some(cost) = cm.ins {
                    f(None, 0, -1, cost, [Some(CigarOp::Insertion), None]);
                }

                // deletion
                if i > 0 && let Some(cost) = cm.del {
                    f(None, -1, 0, cost, [Some(CigarOp::Deletion), None]);
                }

                // affine close
                // NOTE: gap-close edges have a cost of 0 and stay in the current position.
                // This requires that the iteration order over layers at the current position visits the main layer last.
                for (layer, _cml) in cm.affine.iter().enumerate() {
                    f(
                        Some(layer),
                        0,
                        0,
                        0,
                        [Some(CigarOp::AffineClose(layer)), None],
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
                let (i, j, di, dj, op) = match cml.affine_type {
                    AffineLayerType::InsertLayer | AffineLayerType::HomoPolymerInsert => {
                        (i, j - 1, 0, -1, CigarOp::AffineInsertion(layer))
                    }
                    AffineLayerType::DeleteLayer | AffineLayerType::HomoPolymerDelete => {
                        (i - 1, j, -1, 0, CigarOp::AffineDeletion(layer))
                    }
                };
                f(
                    None,
                    di,
                    dj,
                    cml.open + cml.extend,
                    [Some(op), Some(CigarOp::AffineOpen(layer))],
                );

                // gap extend
                if cml.affine_type.is_homopolymer() {
                    // For homopolymer layers, we can only extend if the two characters are equal.
                    if match cml.affine_type {
                        AffineLayerType::HomoPolymerInsert => {
                            j >= 2 && b[j as usize - 1] == b[j as usize - 2]
                        }
                        AffineLayerType::HomoPolymerDelete => {
                            i >= 2 && a[i as usize - 1] == a[i as usize - 2]
                        }
                        _ => unreachable!(),
                    } {
                        f(Some(layer), di, dj, cml.extend, [Some(op), None]);
                    }
                } else {
                    f(Some(layer), di, dj, cml.extend, [Some(op), None]);
                }
            }
        }
    }

    /// Iterate over the states/layers at the given position in 'the right'
    /// order, making sure dependencies within the states at the given position
    /// come first.
    ///
    /// I.e., in this normal case affine layers are iterated before the main
    /// layer, to ensure that the ends of the gap-close edges within this
    /// position are visited first.
    #[inline]
    pub fn iterate_layers<const N: usize>(cm: &AffineCost<N>, mut f: impl FnMut(Layer)) {
        for layer in 0..N {
            f(Some(layer));
        }
        f(None);
    }
}
