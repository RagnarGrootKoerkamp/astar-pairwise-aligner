//! This module contains the `AffineCost` and `LinearCost` cost models.

use std::cmp::{max, min};

/// Type for storing costs. Not u64 to save on memory.
///
/// TODO: Make this a strong type, so that conversion between costs and indices
/// is explicit.
pub type Cost = u32;

/// An affine layer can either correspond to an insertion or deletion.
#[derive(Clone, Copy, PartialEq, Eq)]
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
    HomoPolymerInsert { open_needs_equal: bool },
    HomoPolymerDelete { open_needs_equal: bool },
}
pub use AffineLayerType::*;

/// An affine layer depends on its type, the open cost, and the extend cost.
pub struct AffineLayerCosts {
    pub affine_type: AffineLayerType,
    pub open: Cost,
    pub extend: Cost,
}

/// A full cost model consists of linear substitution/insertion/delete costs,
/// and zero or more (N) affine layers.
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
}

/// A linear cost model is simply an affine cost model without any affine layers.
pub type LinearCost = AffineCost<0>;

impl<const N: usize> AffineCost<N> {
    pub fn new_lcs() -> AffineCost<0> {
        AffineCost {
            sub: None,
            ins: Some(1),
            del: Some(1),
            affine: [],
        }
    }

    pub fn new_unit() -> AffineCost<0> {
        AffineCost {
            sub: Some(1),
            ins: Some(1),
            del: Some(1),
            affine: [],
        }
    }

    pub fn new_linear(sub: Cost, ins: Cost, del: Cost) -> AffineCost<0> {
        AffineCost {
            sub: Some(sub),
            ins: Some(ins),
            del: Some(del),
            affine: [],
        }
    }

    pub fn new_affine(sub: Cost, open: Cost, extend: Cost) -> AffineCost<2> {
        AffineCost {
            sub: Some(sub),
            ins: None,
            del: None,
            affine: [
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
        }
    }

    pub fn new_affine2(
        sub: Cost,
        ins_open: Cost,
        ins_extend: Cost,
        del_open: Cost,
        del_extend: Cost,
    ) -> AffineCost<2> {
        AffineCost {
            sub: Some(sub),
            ins: None,
            del: None,
            affine: [
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

    // Below here are a lot of small helper functions to find the least/most expensive open/extend/open+extend costs.

    pub fn max_ins_open_cost(&self) -> Cost {
        let mut c = Cost::MIN;
        for cm in &self.affine {
            match cm.affine_type {
                InsertLayer | HomoPolymerInsert { .. } => c = max(c, cm.open),
                DeleteLayer | HomoPolymerDelete { .. } => {}
            }
        }
        c
    }

    pub fn max_del_open_cost(&self) -> Cost {
        let mut c = Cost::MIN;
        for cm in &self.affine {
            match cm.affine_type {
                DeleteLayer | HomoPolymerDelete { .. } => c = max(c, cm.open),
                InsertLayer | HomoPolymerInsert { .. } => {}
            }
        }
        c
    }

    pub fn min_ins_extend_cost(&self) -> Cost {
        let mut c = self.ins.unwrap_or(Cost::MAX);
        for cm in &self.affine {
            match cm.affine_type {
                InsertLayer | HomoPolymerInsert { .. } => c = min(c, cm.extend),
                DeleteLayer | HomoPolymerDelete { .. } => {}
            }
        }
        c
    }

    pub fn min_del_extend_cost(&self) -> Cost {
        let mut c = self.del.unwrap_or(Cost::MAX);
        for cm in &self.affine {
            match cm.affine_type {
                DeleteLayer | HomoPolymerDelete { .. } => c = min(c, cm.extend),
                InsertLayer | HomoPolymerInsert { .. } => {}
            }
        }
        c
    }

    pub fn max_ins_extend_cost(&self) -> Cost {
        let mut c = self.ins.unwrap_or(Cost::MIN);
        for cm in &self.affine {
            match cm.affine_type {
                InsertLayer | HomoPolymerInsert { .. } => c = max(c, cm.extend),
                DeleteLayer | HomoPolymerDelete { .. } => {}
            }
        }
        c
    }

    pub fn max_del_extend_cost(&self) -> Cost {
        let mut c = self.del.unwrap_or(Cost::MIN);
        for cm in &self.affine {
            match cm.affine_type {
                DeleteLayer | HomoPolymerDelete { .. } => c = max(c, cm.extend),
                InsertLayer | HomoPolymerInsert { .. } => {}
            }
        }
        c
    }

    pub fn max_ins_open_extend_cost(&self) -> Cost {
        let mut c = self.ins.unwrap_or(0);
        for cm in &self.affine {
            match cm.affine_type {
                InsertLayer | HomoPolymerInsert { .. } => c = max(c, cm.open + cm.extend),
                DeleteLayer | HomoPolymerDelete { .. } => {}
            }
        }
        c
    }

    pub fn max_del_open_extend_cost(&self) -> Cost {
        let mut c = self.del.unwrap_or(0);
        for cm in &self.affine {
            match cm.affine_type {
                DeleteLayer | HomoPolymerDelete { .. } => c = max(c, cm.open + cm.extend),
                InsertLayer | HomoPolymerInsert { .. } => {}
            }
        }
        c
    }
}
