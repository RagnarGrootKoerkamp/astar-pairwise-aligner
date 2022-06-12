//! This module contains various cost models.

use std::cmp::{max, min};

/// Type for costs.
/// TODO: Make this a strong type.
pub type Cost = u32;

pub trait CostModel {
    /// The cost of a substitution.
    /// TODO: Make this depend on the characters being substituted.
    /// Note that this returns an optional, because in case of LCS costs substitutions are not allowed.
    fn sub(&self) -> Option<Cost>;

    /// The cost of a substitution between two specific characters.
    fn sub_cost(&self, a: u8, b: u8) -> Option<Cost> {
        if a == b {
            Some(0)
        } else {
            self.sub()
        }
    }

    /// The cost of inserting a character, or extending an insert.
    fn ins(&self) -> Option<Cost>;

    /// The cost of deleting a character, or extending a deletion.
    fn del(&self) -> Option<Cost>;

    /// Helper functions to deal with the optionals.
    fn sub_or<U, F>(&self, default: U, f: F) -> U
    where
        F: FnOnce(Cost) -> U,
    {
        self.sub().map_or(default, f)
    }

    /// Helper functions to deal with the optionals.
    fn sub_cost_or<U, F>(&self, a: u8, b: u8, default: U, f: F) -> U
    where
        F: FnOnce(Cost) -> U,
    {
        if a == b {
            f(0)
        } else {
            self.sub_or(default, f)
        }
    }

    /// Helper functions to deal with the optionals.
    fn ins_or<U, F>(&self, default: U, f: F) -> U
    where
        F: FnOnce(Cost) -> U,
    {
        self.ins().map_or(default, f)
    }

    /// Helper functions to deal with the optionals.
    fn del_or<U, F>(&self, default: U, f: F) -> U
    where
        F: FnOnce(Cost) -> U,
    {
        self.del().map_or(default, f)
    }

    fn affine(&self) -> &[AffineLayerCosts] {
        &[]
    }

    fn min_ins_open_cost(&self) -> Cost {
        0
    }

    fn min_del_open_cost(&self) -> Cost {
        0
    }

    fn max_ins_open_cost(&self) -> Cost {
        0
    }

    fn max_del_open_cost(&self) -> Cost {
        0
    }

    fn min_ins_extend_cost(&self) -> Cost {
        self.ins().unwrap_or(Cost::MAX)
    }

    fn min_del_extend_cost(&self) -> Cost {
        self.del().unwrap_or(Cost::MAX)
    }

    fn max_ins_extend_cost(&self) -> Cost {
        self.ins().unwrap_or(Cost::MIN)
    }

    fn max_del_extend_cost(&self) -> Cost {
        self.del().unwrap_or(Cost::MIN)
    }

    fn max_ins_open_extend_cost(&self) -> Cost {
        self.ins().unwrap_or(0)
    }

    fn max_del_open_extend_cost(&self) -> Cost {
        self.del().unwrap_or(0)
    }
}

#[derive(Clone, Copy)]
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

pub struct AffineLayerCosts {
    pub affine_type: AffineLayerType,
    pub open: Cost,
    pub extend: Cost,
}

/// N is the number of affine layers.
pub struct AffineCost<const N: usize> {
    pub sub: Option<Cost>,
    pub ins: Option<Cost>,
    pub del: Option<Cost>,
    pub affine: [AffineLayerCosts; N],
}

pub type LinearCost = AffineCost<0>;

impl<const N: usize> CostModel for AffineCost<N> {
    fn sub(&self) -> Option<Cost> {
        self.sub
    }

    fn ins(&self) -> Option<Cost> {
        self.ins
    }

    fn del(&self) -> Option<Cost> {
        self.del
    }

    fn affine(&self) -> &[AffineLayerCosts] {
        &self.affine[..]
    }

    fn min_ins_open_cost(&self) -> Cost {
        let mut c = Cost::MAX;
        for cm in &self.affine {
            match cm.affine_type {
                InsertLayer | HomoPolymerInsert { .. } => c = min(c, cm.open),
                DeleteLayer | HomoPolymerDelete { .. } => {}
            }
        }
        c
    }

    fn min_del_open_cost(&self) -> Cost {
        let mut c = Cost::MAX;
        for cm in &self.affine {
            match cm.affine_type {
                DeleteLayer | HomoPolymerDelete { .. } => c = min(c, cm.open),
                InsertLayer | HomoPolymerInsert { .. } => {}
            }
        }
        c
    }

    fn max_ins_open_cost(&self) -> Cost {
        let mut c = Cost::MIN;
        for cm in &self.affine {
            match cm.affine_type {
                InsertLayer | HomoPolymerInsert { .. } => c = max(c, cm.open),
                DeleteLayer | HomoPolymerDelete { .. } => {}
            }
        }
        c
    }

    fn max_del_open_cost(&self) -> Cost {
        let mut c = Cost::MIN;
        for cm in &self.affine {
            match cm.affine_type {
                DeleteLayer | HomoPolymerDelete { .. } => c = max(c, cm.open),
                InsertLayer | HomoPolymerInsert { .. } => {}
            }
        }
        c
    }

    fn min_ins_extend_cost(&self) -> Cost {
        let mut c = self.ins().unwrap_or(Cost::MAX);
        for cm in &self.affine {
            match cm.affine_type {
                InsertLayer | HomoPolymerInsert { .. } => c = min(c, cm.extend),
                DeleteLayer | HomoPolymerDelete { .. } => {}
            }
        }
        c
    }

    fn min_del_extend_cost(&self) -> Cost {
        let mut c = self.del().unwrap_or(Cost::MAX);
        for cm in &self.affine {
            match cm.affine_type {
                DeleteLayer | HomoPolymerDelete { .. } => c = min(c, cm.extend),
                InsertLayer | HomoPolymerInsert { .. } => {}
            }
        }
        c
    }

    fn max_ins_extend_cost(&self) -> Cost {
        let mut c = self.ins().unwrap_or(Cost::MIN);
        for cm in &self.affine {
            match cm.affine_type {
                InsertLayer | HomoPolymerInsert { .. } => c = max(c, cm.extend),
                DeleteLayer | HomoPolymerDelete { .. } => {}
            }
        }
        c
    }

    fn max_del_extend_cost(&self) -> Cost {
        let mut c = self.del().unwrap_or(Cost::MIN);
        for cm in &self.affine {
            match cm.affine_type {
                DeleteLayer | HomoPolymerDelete { .. } => c = max(c, cm.extend),
                InsertLayer | HomoPolymerInsert { .. } => {}
            }
        }
        c
    }

    fn max_ins_open_extend_cost(&self) -> Cost {
        let mut c = self.ins().unwrap_or(0);
        for cm in &self.affine {
            match cm.affine_type {
                InsertLayer | HomoPolymerInsert { .. } => c = max(c, cm.open + cm.extend),
                DeleteLayer | HomoPolymerDelete { .. } => {}
            }
        }
        c
    }

    fn max_del_open_extend_cost(&self) -> Cost {
        let mut c = self.del().unwrap_or(0);
        for cm in &self.affine {
            match cm.affine_type {
                DeleteLayer | HomoPolymerDelete { .. } => c = max(c, cm.open + cm.extend),
                InsertLayer | HomoPolymerInsert { .. } => {}
            }
        }
        c
    }
}

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
}
