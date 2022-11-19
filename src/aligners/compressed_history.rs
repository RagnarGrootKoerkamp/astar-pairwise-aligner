//! Instead of storing each visited state `(d, fr)`, it is sufficient for
//! path reconstruction to only store those states `(d, fr)` that:
//! - have a child reached via substitution.
//!
//!
//! For all states on the last front, we store their last stored parent.
//!
//! - Remark the following:
//!   - a substitution can never be followed by preceded by another error, since the parent of each substitution edge is stored,
//!   - we assume that an insertion followed by a deletion is never optimal, i.e.:
//!     NOTE: ASSUMPTION: sub_cost <= min_insert_extend + min_delete_extend.
//! - From this we conclude:
//!   - FIXME: the regex is false
//!     The errors on a path between any two states look like this regex:
//!     S?([ID]+S+), i.e.: substitutions interleaved with runs on insertions
//!     or deletions.
//!   - Since we store the parent of each substitution, the path to the stored parent can never have two substitutions.
//!   - CONCLUSION: The path to the stored parent contains only insertions or only deletions, followed by at most one substitution.
//!
//! For any visited state, the path to its parent can now be inferred like this:
//! - First, do greedy backwards matching, and do greedy matching again after each error.
//! - If the stored parent is on the same diagonal, it must be a direct parent.
//! - If the stored parent is on a higher diagonal, there must be insertions on the path there.
//!   Substitutions can not follow insertions, so the direct parent is via insertion.
//! - If the stored parent is on a lower diagonal, there must be deletions on the path there.
//!   Substitutions can not follow insertions, so the direct parent is via insertion.
//!
//! When there are (multiple) affine layers, this changes as follows:
//! - Main layer states are also stored when they have an outgoing gap-open edge that is used (optimal for the affine layer child).
//! - Affine states are also stored when their outgoing gap-close edge is used (optimal for the main layer child).
//! - Each (parent) state now also contains the layer.
//! - When tracing back, the following changes:
//!   - If in affine layer currently: jump to the stored parent, which must
//!     be the state from which the affine gap is opened.
//!   - If in main layer currently, and the stored parent is in affine
//!     layer: trace to the direct main-layer child of the stored parent, and
//!     then enter the affine layer.
//!
//! TODO: Think about the following cases:
//! - A path can have insert - matches - delete, without intermediate
//!   substitution. In this case, we must still store the state where the deletion
//!   started.
//! - Do we need to store both affine open and affine close states?
//!   - Affine close may not be needed as long as we know which affine layer to enter, and at which point to enter it.
//!     We should probably do linear indels/matches until at least the
//!     row/column of the affine indel state, and then (after possibly a bit
//!     more greedy matching) enter it.

use super::{cigar::Cigar, diagonal_transition::Fr, Seq};
use crate::{aligners::cigar::CigarOp, prelude::Pos};

/// Contains the diagonal `d` and furthest reaching value `fr` and `layer`.
#[derive(Clone, PartialEq, Eq, Copy)]
pub struct TracebackState {
    pub d: Fr,
    pub fr: Fr,
    /// TODO: Use `NonMax<u8>`?
    pub layer: Option<usize>,
}

impl TracebackState {
    pub fn to_coords(&self) -> (Fr, Fr) {
        ((self.fr + self.d) / 2, (self.fr - self.d) / 2)
    }

    pub fn to_coords_u(&self) -> (usize, usize) {
        assert!(self.d <= self.fr && -self.d <= self.fr);
        let (i, j) = self.to_coords();
        (i as usize, j as usize)
    }

    pub fn to_pos(&self) -> crate::prelude::Pos {
        let (i, j) = self.to_coords_u();
        Pos::from(i, j)
    }

    pub fn from_pos(Pos(i, j): Pos) -> Self {
        Self {
            d: i as Fr - j as Fr,
            fr: i as Fr + j as Fr,
            layer: None,
        }
    }

    pub fn root() -> Self {
        Self {
            d: 0,
            fr: 0,
            layer: None,
        }
    }

    pub fn target(a: Seq, b: Seq) -> Self {
        Self::from_pos(Pos::from_lengths(a, b))
    }
}

/// Each state is referred to by its index in the vector.
/// NOTE that this uses a private field, to keep things simple for users.
/// TODO: Use `NonMax<u32>`?
#[derive(Clone, Copy)]
pub struct StateId(usize);

pub struct CompressedHistory<'s> {
    a: Seq<'s>,
    b: Seq<'s>,
    /// For each state, we store the state itself and the id of its parent.
    states: Vec<(Option<StateId>, TracebackState)>,
}

impl<'s> CompressedHistory<'s> {
    pub fn new(a: Seq<'s>, b: Seq<'s>) -> Self {
        Self {
            a,
            b,
            states: vec![(None, TracebackState::root())],
        }
    }
}

impl<'s> CompressedHistory<'s> {
    pub fn root_id(&self) -> StateId {
        StateId(0)
    }

    /// Adds the given state with the given parent, and returns the id of the pushed state.
    pub fn push(&mut self, state: TracebackState, parent_id: StateId) -> StateId {
        self.states.push((Some(parent_id), state));
        StateId(self.states.len() - 1)
    }

    /// The id of the parent, or `None` for the root.
    pub fn parent(&self, state_id: StateId) -> Option<StateId> {
        self.states[state_id.0].0
    }

    /// Get the stored state for an id.
    pub fn get(&self, state_id: StateId) -> &TracebackState {
        &self.states[state_id.0].1
    }

    /// Trace to the direct parent of the current position.
    /// The actual sequences `a` and `b` are needed to do greedy matching in main layer states.
    pub fn parent_state(
        &self,
        mut cur: TracebackState,
        parent_id: StateId,
        cigar: Option<&mut Cigar>,
    ) -> (TracebackState, Option<StateId>) {
        let parent = self.get(parent_id);

        // The number of diagonals we need to change to get to parent.
        let dd = parent.d - cur.d;

        // 1, in the direction of the parent.
        let d_unit = if dd > 0 { 1 } else { -1 };

        if let Some(layer) = cur.layer {
            // In affine layer, the parent must be the state from where the gap was opened.
            assert!(parent.layer.is_none());

            // TODO: Generalize this to a EditGraph template parameter.
            // For now, we assume gap-open costs.

            let cigarop = if dd > 0 {
                CigarOp::AffineInsertion(layer)
            } else {
                CigarOp::AffineDeletion(layer)
            };

            match dd {
                dd if dd > 1 || dd < -1 => {
                    // extend insert/delete
                    cur.d += d_unit;
                    cur.fr -= 1;
                    if let Some(cigar) = cigar {
                        cigar.push(cigarop);
                    }
                    (cur, Some(parent_id))
                }
                dd if dd == 1 || dd == -1 => {
                    // open insert/delete
                    cur.d += d_unit;
                    cur.layer = None;
                    cur.fr -= 1;
                    assert!(cur.d == parent.d);
                    assert!(cur.layer == parent.layer);
                    assert!(cur.fr <= parent.fr);
                    if let Some(cigar) = cigar {
                        cigar.push(cigarop);
                        cigar.push(CigarOp::AffineOpen(layer));
                    }
                    (cur, self.parent(parent_id))
                }
                0 => panic!(),
                _ => unreachable!(),
            }
        } else {
            // In the main layer, there are a two options for the parent:
            // - a substitution parent,
            // - affine gap-close parent.
            //
            // In both cases, we may first need to walk inside the main layer. This consists of alternating:
            // - greedy matching
            // - a linear indel in the right direction.

            // 1. Greedy matching is always allowed.
            // NOTE: Doing greedy matching before entering an affine layer, will
            // break the assumption that we enter the affine layer at exactly
            // the given parent state.
            let (i, j) = cur.to_coords_u();
            if i > 0 && j > 0 && self.a[i - 1] == self.b[j - 1] {
                // greedy match
                cur.fr -= 2;
                //assert!(cur.fr >= parent.fr);
                if let Some(cigar) = cigar {
                    cigar.push(CigarOp::Match);
                }
                return (cur, Some(parent_id));
            }

            // 2. If not in diagonal of parent, make a linear indel in that direction.
            if dd != 0 {
                // linear insert / delete
                cur.d += d_unit;
                cur.fr -= 1;
                assert!(cur.fr >= parent.fr);
                if let Some(cigar) = cigar {
                    cigar.push(if dd > 0 {
                        CigarOp::Insertion
                    } else {
                        CigarOp::Deletion
                    });
                }
                return (cur, Some(parent_id));
            }

            // 3. We are in the right diagonal
            if let Some(parent_layer) = parent.layer {
                // affine close
                // s, d, and fr do not change.
                cur.layer = Some(parent_layer);
                if let Some(cigar) = cigar {
                    cigar.push(CigarOp::AffineClose(parent_layer));
                }
                return (cur, self.parent(parent_id));
            } else {
                // substitution
                // d does not change
                cur.fr -= 2;
                assert!(cur == *parent);
                if let Some(cigar) = cigar {
                    cigar.push(CigarOp::Sub);
                }
                return (cur, self.parent(parent_id));
            }
            // Unreachable.
        }
    }

    pub fn traceback(&self, mut state: TracebackState, parent: StateId) -> Cigar {
        let mut cigar = Cigar::default();
        let mut parent = Some(parent);
        while let Some(p) = parent {
            (state, parent) = self.parent_state(state, p, Some(&mut cigar));
        }
        cigar
    }
}
