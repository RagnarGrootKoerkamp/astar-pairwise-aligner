//! `EditGraph` is an abstraction for the graph corresponding to a cost model.
//! There may be multiple graphs corresponding to the same cost model:
//! https://research.curiouscoding.nl/posts/diagonal-transition-variations/

use super::Seq;
use crate::cost_model::{AffineCost, AffineLayerType, Cost};
use std::cmp::min;

/// TODO: Generalize to CostModel trait, instead of only AffineCost.
/// TODO: Make EditGraph a trait instead of type? Then we can implement
/// different graph shapes in different types, and each can be optimized individually.
/// TODO: Decide whether this is a type only class that takes all arguments on
/// each invocation, or whether it owns the sequences and cost model, and all
/// inspection of them has to go through it.
pub struct EditGraph<'seq, 'cm, const N: usize> {
    pub a: Seq<'seq>,
    pub b: Seq<'seq>,
    pub cm: &'cm AffineCost<N>,

    /// When true, if there is a match (in the main/linear layer), all other edges are skipped.
    /// TODO: Make template argument.
    pub greedy_matching: bool,
}

pub type Layer = Option<usize>;

pub struct State {
    pub i: I,
    pub j: I,
    pub layer: Layer,
}

impl State {
    #[inline]
    pub fn new(i: I, j: I, layer: Layer) -> Self {
        Self { i, j, layer }
    }
}

type I = isize;

impl<'seq, 'cm, const N: usize> EditGraph<'seq, 'cm, N> {
    /// Iterate over the parents of the given state by calling `f` for each of them.
    /// Parents of a state are closer to (0,0) that the state itself.
    ///
    /// `iterate_children` may be needed as well at some point, but currently we
    /// use a 'pull-based' DP, meaning that in each state we look back, and
    /// never 'push' to the children.
    ///
    /// TODO: Add CigarOp to `f` argument?
    #[inline]
    pub fn iterate_parents(&self, state: State, f: impl FnMut(Layer, I, I, Cost)) {
        self.iterate_parents_internal(state, f, true)
    }

    /// This function is slightly more generic, in that we can disable
    /// iterations over edges that stay at the given position.
    #[inline]
    pub fn iterate_parents_internal(
        &self,
        State { i, j, layer }: State,
        mut f: impl FnMut(Layer, I, I, Cost),
        include_in_layer_edges: bool,
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
                let is_match = self.a[i as usize - 1] == self.b[j as usize - 1];
                if is_match {
                    f(None, -1, -1, 0);
                    if self.greedy_matching {
                        return;
                    }
                } else {
                    f(None, -1, -1, self.cm.sub.unwrap_or(Cost::MAX / 2));
                }

                // insertion
                if j > 0 && let Some(cost) = self.cm.ins {
                    f(None, 0, -1, cost);
                }

                // deletion
                if i > 0 && let Some(cost) = self.cm.del {
                    f(None, -1, 0, cost);
                }

                // affine close
                if include_in_layer_edges {
                    // NOTE: gap-close edges have a cost of 0 and stay in the current position.
                    // This requires that the iteration order over layers at the current position visits the main layer last.
                    for (layer, _cml) in self.cm.affine.iter().enumerate() {
                        f(Some(layer), 0, 0, 0);
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
                let cml = &self.cm.affine[layer];
                let (i, j, di, dj) = match cml.affine_type {
                    AffineLayerType::InsertLayer | AffineLayerType::HomoPolymerInsert => {
                        (i, j - 1, 0, -1)
                    }
                    AffineLayerType::DeleteLayer | AffineLayerType::HomoPolymerDelete => {
                        (i - 1, j, -1, 0)
                    }
                };
                f(None, di, dj, cml.open + cml.extend);

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
                        f(Some(layer), di, dj, cml.extend);
                    }
                } else {
                    f(Some(layer), di, dj, cml.extend);
                }
            }
        }
    }

    /// A wrapper over `iterate_parents` that keeps a running minimum.
    #[inline]
    pub fn minimum_over_parents<T>(
        &self,
        state: State,
        cost_of_edge: &mut impl FnMut(Layer, I, I, Cost) -> T,
        init: T,
    ) -> T
    where
        T: Ord,
    {
        let mut t = init;
        self.iterate_parents(state, |parent_state, di, dj, cost| {
            let parent_t = cost_of_edge(parent_state, di, dj, cost);
            if parent_t < t {
                t = parent_t;
            }
        });
        t
    }

    /// Iterate over the states/layers at the given position in 'the right'
    /// order, making sure dependencies within the states at the given position
    /// come first.
    ///
    /// I.e., in this normal case affine layers are iterated before the main
    /// layer, to ensure that the ends of the gap-close edges within this
    /// position are visited first.
    #[inline]
    pub fn iterate_layers(&self, mut f: impl FnMut(Layer)) {
        for layer in 0..N {
            f(Some(layer));
        }
        f(None);
    }

    /// This function is similar to the one above, but iterates through all
    /// states/layers at the given position, and expands any edges that
    /// depend on this position itself. In the default case, the main layer depends on
    /// the gap-close edges in the affine layers. We want to emulate the following:
    /// ```
    /// let mut f = INF;
    /// for layer in affine_layer {
    ///     cost_for_layer = edit_graph.iterate_parents(..);
    ///     f = min(f, cost_for_layer);
    /// }
    /// cost_for_main_layer = f;
    /// ```
    ///
    /// There are two way to handle such edges:
    /// 1. Make sure that the dependent layers are visited/computed first, and
    ///    then simply iterate over the edges in the layers that depend on them.
    /// 2. 'Expand' the in-pos edges, remembering the 'best' value for each of them, and re-using these for the main layer.
    ///
    /// The second option is chosen here. For the first option, use `iterate_layers`.
    #[inline]
    pub fn iterate_parents_of_position<T>(
        &self,
        i: I,
        j: I,
        // The cost of taking an edge.
        mut cost_of_edge: impl FnMut(Layer, I, I, Cost) -> T,
        init: T,
    ) -> (T, [T; N])
    where
        T: Ord + Copy,
    {
        let mut main_cost = init;
        let mut affine_cost = [init; N];

        // In our case, we first iterate over all affine layers, keeping a running minimum of `t`.
        // For the final layer, we use both this `t` and the remaining non-affine edges.
        for layer in 0..N {
            affine_cost[layer] =
                self.minimum_over_parents(State::new(i, j, Some(layer)), &mut cost_of_edge, init);
            main_cost = min(main_cost, affine_cost[layer]);
        }
        main_cost = min(
            main_cost,
            self.minimum_over_parents(State::new(i, j, None), &mut cost_of_edge, init),
        );
        (main_cost, affine_cost)
    }
}
