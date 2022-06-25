//! `EditGraph` is an abstraction for the graph corresponding to a cost model.
//! There may be multiple graphs corresponding to the same cost model:
//! https://research.curiouscoding.nl/posts/diagonal-transition-variations/

use super::{cigar::CigarOp, Seq};
use crate::{
    cost_model::{AffineCost, AffineLayerType, Cost},
    prelude::Pos,
};

/// TODO: Generalize to CostModel trait, instead of only AffineCost.
/// TODO: Make EditGraph a trait instead of type? Then we can implement
/// different graph shapes in different types, and each can be optimized individually.
pub struct EditGraph<'seq, const N: usize> {
    a: Seq<'seq>,
    b: Seq<'seq>,
    cm: AffineCost<N>,

    /// When true, if there is a match (in the main/linear layer), all other edges are skipped.
    /// TODO: Make template argument.
    greedy_matching: bool,
}

pub type Layer = Option<usize>;

pub struct State(Pos, Layer);

impl<'seq, const N: usize> EditGraph<'seq, N> {
    /// Iterate over the parents of the given state by calling `f` for each of them.
    /// Parents of a state are closer to (0,0) that the state itself.
    ///
    /// `iterate_children` may be needed as well at some point, but currently we
    /// use a 'pull-based' DP, meaning that in each state we look back, and
    /// never 'push' to the children.
    ///
    /// TODO: Add CigarOp to `f` argument?
    pub fn iterate_parents(&self, State(Pos(i, j), layer): State, mut f: impl FnMut(State, Cost)) {
        match layer {
            None => {
                // In the main layer, there are many possible edges:
                // - match
                // - mismatch / substitution
                // - insertion
                // - deletion
                // - affine close (insertion or deletion)

                // match / mismatch
                if i > 0 && j > 0 {
                    let is_match = self.a[i as usize - 1] == self.b[j as usize - 1];
                    if is_match {
                        f(State(Pos(i - 1, j - 1), None), 0);
                        if self.greedy_matching {
                            return;
                        }
                    } else if let Some(cost) = self.cm.sub {
                        f(State(Pos(i - 1, j - 1), None), cost);
                    }
                }

                // insertion
                if j > 0 && let Some(cost) = self.cm.ins {
                    f(State(Pos(i, j - 1), None), cost);
                }

                // deletion
                if i > 0 && let Some(cost) = self.cm.del {
                    f(State(Pos(i-1, j), None), cost);
                }

                // affine close
                for (layer, _cml) in self.cm.affine.iter().enumerate() {
                    // NOTE: gap-close cost is 0 in the default model.
                    f(State(Pos(i, j), Some(layer)), 0);
                }
            }
            Some(layer) => {
                // If we are currently in an affine layer, there are two options:
                // - gap open of cost `open + extend`, which is always allowed.
                // - gap close of cost `extend`, which in case of homopolymer
                //   layers is only allowed when the character to be extended
                //   equals the previous character.

                // gap open
                let cml = &self.cm.affine[layer];
                let parent_pos = match cml.affine_type {
                    AffineLayerType::InsertLayer | AffineLayerType::HomoPolymerInsert => {
                        Pos(i, j - 1)
                    }
                    AffineLayerType::DeleteLayer | AffineLayerType::HomoPolymerDelete => {
                        Pos(i - 1, j)
                    }
                };
                f(State(parent_pos, None), cml.open + cml.extend);

                // gap extend
                if cml.affine_type.is_homopolymer() {
                    // For homopolymer layers, we can only extend if the two characters are equal.
                    if match cml.affine_type {
                        AffineLayerType::HomoPolymerInsert => {
                            j >= 2 && self.b[j as usize - 1] == self.b[j as usize - 2]
                        }
                        AffineLayerType::HomoPolymerDelete => {
                            i >= 2 && self.a[i as usize - 1] == self.a[i as usize - 2]
                        }
                        _ => unreachable!(),
                    } {
                        f(State(parent_pos, Some(layer)), cml.extend);
                    }
                } else {
                    f(State(parent_pos, Some(layer)), cml.extend);
                }
            }
        }
    }
}
