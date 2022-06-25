//! This module contains the `AffineCost` and `LinearCost` cost models.
//!
use std::cmp::{max, min};

use crate::{
    aligners::cigar::CigarOp,
    prelude::{Pos, I},
};

/// Type for storing costs. Not u64 to save on memory.
///
/// TODO: Make this a strong type, so that conversion between costs and indices
/// is explicit.
pub type Cost = u32;

/// A trait describing a cost model.
///
/// This is currently only implemented for `AffineCost<N>`, but may be
/// specialized for e.g. `UnitCost` to allow better compile time optimizations.
pub trait CostModel {
    /// The minimal cost according tho this cost model to go from one position to another.
    fn gap_cost(&self, s: Pos, t: Pos) -> Cost;
}

/// An affine layer can either correspond to an insertion or deletion.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AffineLayerType {
    InsertLayer,
    DeleteLayer,
    // TODO: Add homopolymer affine layers that only allow inserting/deleting duplicate characters.
    // I.e.:
    // abc -> abbbc
    // abbbc -> abc
    // but not:
    // ac -> abbbc
    // abbbc -> ac
    //
    // TODO:
    // We could also decide to allow this last example, where the run of
    // inserted/deleted characters has to be equal, but does not have to be
    // equal to any adjacent character. However, that will likely cover more
    // unintended single-character mutations.
    // We could make this a parameter of the enum variant.
    HomoPolymerInsert,
    HomoPolymerDelete,
}

pub use AffineLayerType::*;

/// An affine layer depends on its type, the open cost, and the extend cost.
#[derive(Clone)]
pub struct AffineLayerCosts {
    pub affine_type: AffineLayerType,
    pub open: Cost,
    pub extend: Cost,
}

/// A full cost model consists of linear substitution/insertion/delete costs,
/// and zero or more (N) affine layers.
#[derive(Clone)]
pub struct AffineCost<const N: usize> {
    /// The substitution cost. Or None when substitutions are not allowed.
    pub sub: Option<Cost>,
    /// The insertion cost. Or None when substitutions are not allowed.
    pub ins: Option<Cost>,
    /// The deletion cost. Or None when substitutions are not allowed.
    pub del: Option<Cost>,
    /// `N` affine layers.
    /// TODO: Should we split this into `NI` insertion layers and `ND` deletion
    /// layers, so that matching on the type becomes a compile-time instead of
    /// run-time operation?
    pub affine: [AffineLayerCosts; N],

    /// Extra fields derived from the affine layers.
    /// We store them so we do not have to recompute them all the time.
    pub min_ins_open: Cost,
    pub max_ins_open: Cost,
    pub min_del_open: Cost,
    pub max_del_open: Cost,
    pub min_ins_extend: Cost,
    pub max_ins_extend: Cost,
    pub min_del_extend: Cost,
    pub max_del_extend: Cost,
    pub min_ins_open_extend: Cost,
    pub max_ins_open_extend: Cost,
    pub min_del_open_extend: Cost,
    pub max_del_open_extend: Cost,
}

/// A linear cost model is simply an affine cost model without any affine layers.
pub type LinearCost = AffineCost<0>;

impl LinearCost {
    pub fn new_lcs() -> LinearCost {
        Self::new(None, Some(1), Some(1), [])
    }

    pub fn new_unit() -> LinearCost {
        Self::new(Some(1), Some(1), Some(1), [])
    }

    pub fn new_linear(sub: Cost, indel: Cost) -> LinearCost {
        Self::new(Some(sub), Some(indel), Some(indel), [])
    }

    pub fn new_linear_asymmetric(sub: Cost, ins: Cost, del: Cost) -> LinearCost {
        Self::new(Some(sub), Some(ins), Some(del), [])
    }
}

impl AffineCost<2> {
    pub fn new_affine(sub: Cost, open: Cost, extend: Cost) -> AffineCost<2> {
        Self::new(
            Some(sub),
            None,
            None,
            [
                AffineLayerCosts {
                    affine_type: InsertLayer,
                    open,
                    extend,
                },
                AffineLayerCosts {
                    affine_type: DeleteLayer,
                    open,
                    extend,
                },
            ],
        )
    }
    pub fn new_linear_affine(sub: Cost, indel: Cost, open: Cost, extend: Cost) -> AffineCost<2> {
        Self::new(
            Some(sub),
            Some(indel),
            Some(indel),
            [
                AffineLayerCosts {
                    affine_type: InsertLayer,
                    open,
                    extend,
                },
                AffineLayerCosts {
                    affine_type: DeleteLayer,
                    open,
                    extend,
                },
            ],
        )
    }
    pub fn new_affine_asymmetric(
        sub: Cost,
        ins_open: Cost,
        ins_extend: Cost,
        del_open: Cost,
        del_extend: Cost,
    ) -> AffineCost<2> {
        Self::new(
            Some(sub),
            None,
            None,
            [
                AffineLayerCosts {
                    affine_type: InsertLayer,
                    open: ins_open,
                    extend: ins_extend,
                },
                AffineLayerCosts {
                    affine_type: DeleteLayer,
                    open: del_open,
                    extend: del_extend,
                },
            ],
        )
    }
}
impl AffineCost<4> {
    pub fn new_double_affine(
        sub: Cost,
        open: Cost,
        extend: Cost,
        open2: Cost,
        extend2: Cost,
    ) -> AffineCost<4> {
        Self::new(
            Some(sub),
            None,
            None,
            [
                AffineLayerCosts {
                    affine_type: InsertLayer,
                    open,
                    extend,
                },
                AffineLayerCosts {
                    affine_type: DeleteLayer,
                    open,
                    extend,
                },
                AffineLayerCosts {
                    affine_type: InsertLayer,
                    open: open2,
                    extend: extend2,
                },
                AffineLayerCosts {
                    affine_type: DeleteLayer,
                    open: open2,
                    extend: extend2,
                },
            ],
        )
    }
}

impl<const N: usize> AffineCost<N> {
    pub fn new(
        sub: Option<Cost>,
        ins: Option<Cost>,
        del: Option<Cost>,
        affine: [AffineLayerCosts; N],
    ) -> AffineCost<N> {
        let layers = |affine_type| {
            affine
                .iter()
                .filter(move |cm| cm.affine_type == affine_type)
        };
        let min_by = |affine_type, f: &dyn Fn(&AffineLayerCosts) -> Cost| {
            let mut c = layers(affine_type).map(f).min().unwrap_or(Cost::MAX);
            // Also include the linear layer in the affine maximums.
            if let Some(extend) = if affine_type == AffineLayerType::InsertLayer {
                ins
            } else {
                del
            } {
                c = min(
                    c,
                    f(&AffineLayerCosts {
                        affine_type,
                        open: 0,
                        extend,
                    }),
                );
            }
            c
        };
        let max_by = |affine_type, f: &dyn Fn(&AffineLayerCosts) -> Cost| {
            let mut c = layers(affine_type).map(f).max().unwrap_or(Cost::MIN);
            // Also include the linear layer in the affine maximums.
            if let Some(extend) = if affine_type == AffineLayerType::InsertLayer {
                ins
            } else {
                del
            } {
                c = max(
                    c,
                    f(&AffineLayerCosts {
                        affine_type,
                        open: 0,
                        extend,
                    }),
                );
            }
            c
        };
        let min_ins_open = min_by(InsertLayer, &|cm| cm.open);
        let max_ins_open = max_by(InsertLayer, &|cm| cm.open);
        let min_del_open = min_by(DeleteLayer, &|cm| cm.open);
        let max_del_open = max_by(DeleteLayer, &|cm| cm.open);
        let min_ins_extend = min_by(InsertLayer, &|cm| cm.extend);
        let max_ins_extend = max_by(InsertLayer, &|cm| cm.extend);
        let min_del_extend = min_by(DeleteLayer, &|cm| cm.extend);
        let max_del_extend = max_by(DeleteLayer, &|cm| cm.extend);
        let min_ins_open_extend = min_by(InsertLayer, &|cm| cm.open + cm.extend);
        let max_ins_open_extend = max_by(InsertLayer, &|cm| cm.open + cm.extend);
        let min_del_open_extend = min_by(DeleteLayer, &|cm| cm.open + cm.extend);
        let max_del_open_extend = max_by(DeleteLayer, &|cm| cm.open + cm.extend);
        AffineCost {
            sub,
            ins,
            del,
            affine,
            min_ins_open,
            max_ins_open,
            min_del_open,
            max_del_open,
            min_ins_extend,
            max_ins_extend,
            min_del_extend,
            max_del_extend,
            min_ins_open_extend,
            max_ins_open_extend,
            min_del_open_extend,
            max_del_open_extend,
        }
    }

    pub fn sub_cost(&self, a: u8, b: u8) -> Option<Cost> {
        if a == b {
            Some(0)
        } else {
            {
                let ref this = self;
                this.sub
            }
        }
    }

    pub fn sub_or<U, F>(&self, default: U, f: F) -> U
    where
        F: FnOnce(Cost) -> U,
    {
        self.sub.map_or(default, f)
    }

    pub fn sub_cost_or<U, F>(&self, a: u8, b: u8, default: U, f: F) -> U
    where
        F: FnOnce(Cost) -> U,
    {
        if a == b {
            f(0)
        } else {
            self.sub_or(default, f)
        }
    }

    pub fn ins_or<U, F>(&self, default: U, f: F) -> U
    where
        F: FnOnce(Cost) -> U,
    {
        self.ins.map_or(default, f)
    }

    pub fn del_or<U, F>(&self, default: U, f: F) -> U
    where
        F: FnOnce(Cost) -> U,
    {
        self.del.map_or(default, f)
    }

    /// NOTE: This also includes the linear insert cost.
    pub fn for_ins(&self, mut f: impl FnMut(Cost, Cost)) {
        if let Some(ins) = self.ins {
            f(0, ins);
        }
        for cm in &self.affine {
            if cm.affine_type == AffineLayerType::InsertLayer {
                f(cm.open, cm.extend);
            }
        }
    }

    /// NOTE: This also includes the linear delete cost.
    pub fn for_del(&self, mut f: impl FnMut(Cost, Cost)) {
        if let Some(del) = self.del {
            f(0, del);
        }
        for cm in &self.affine {
            if cm.affine_type == AffineLayerType::DeleteLayer {
                f(cm.open, cm.extend);
            }
        }
    }

    /// Returns 0 when insertions are not possible.
    pub fn max_ins_for_cost(&self, s: Cost) -> I {
        let mut d = 0;
        self.for_ins(|o, e| d = max(d, s.saturating_sub(o) / e));
        d
    }

    /// Returns 0 when deletions are not possible.
    pub fn max_del_for_cost(&self, s: Cost) -> I {
        let mut d = 0;
        self.for_del(|o, e| d = max(d, s.saturating_sub(o) / e));
        d
    }

    /// d<0: insertion cost
    /// d=0: substitution cost
    /// d>0: deletion cost
    pub fn linear_cost_in_direction(&self, d: i32) -> Option<Cost> {
        match d {
            d if d < 0 => self.ins,
            d if d == 0 => self.sub,
            d if d > 0 => self.del,
            _ => unreachable!(),
        }
    }

    pub fn to_cigar(&self, layer: usize) -> CigarOp {
        match self.affine[layer].affine_type {
            InsertLayer | HomoPolymerInsert => CigarOp::AffineInsertion(layer),
            DeleteLayer | HomoPolymerDelete => CigarOp::AffineDeletion(layer),
        }
    }
}

impl<const N: usize> CostModel for AffineCost<N> {
    fn gap_cost(&self, s: Pos, t: Pos) -> Cost {
        let delta = (t.0 - s.0) as isize - (t.1 - s.1) as isize;
        match delta {
            0 => 0,
            d if d < 0 => {
                let d = (-d) as Cost;
                let mut c = Cost::MAX;
                if let Some(ins) = self.ins {
                    c = min(c, d * ins);
                }
                for cm in &self.affine {
                    if cm.affine_type == AffineLayerType::InsertLayer {
                        c = min(c, cm.open + d * cm.extend);
                    }
                }
                assert!(c != Cost::MAX);
                c
            }
            d if d > 0 => {
                let d = d as Cost;
                let mut c = Cost::MAX;
                if let Some(del) = self.del {
                    c = min(c, d * del);
                }
                for cm in &self.affine {
                    if cm.affine_type == AffineLayerType::DeleteLayer {
                        c = min(c, cm.open + d * cm.extend);
                    }
                }
                assert!(c != Cost::MAX);
                c
            }
            _ => unreachable!(),
        }
    }
}
