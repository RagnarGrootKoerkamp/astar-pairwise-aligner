//! This module contains various cost models.

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

    // FIXME: Add methods for open cost.
}

/// Implement this trait to indicate that the cost model does not use affine costs.
pub trait LinearCostModel: CostModel {}

pub struct LinearCost {
    /// The substitution cost. None for LCS where substitutions are not allowed.
    pub sub: Option<Cost>,
    pub ins: Option<Cost>,
    pub del: Option<Cost>,
}

impl CostModel for LinearCost {
    fn sub(&self) -> Option<Cost> {
        self.sub
    }

    fn ins(&self) -> Option<Cost> {
        self.ins
    }

    fn del(&self) -> Option<Cost> {
        self.del
    }
}
impl LinearCostModel for LinearCost {}

impl LinearCost {
    pub fn lcs() -> Self {
        Self {
            sub: None,
            ins: Some(1),
            del: Some(1),
        }
    }
    pub fn unit() -> Self {
        Self {
            sub: Some(1),
            ins: Some(1),
            del: Some(1),
        }
    }
    pub fn edit_cost(sub: Cost, indel: Cost) -> Self {
        Self {
            sub: Some(sub),
            ins: Some(indel),
            del: Some(indel),
        }
    }
    pub fn edit_cost2(sub: Cost, ins: Cost, del: Cost) -> Self {
        Self {
            sub: Some(sub),
            ins: Some(ins),
            del: Some(del),
        }
    }
}

pub enum AffineLayerType {
    Insert,
    Delete,
    // TODO: Add homopolymer affine layers that only allow inserting/deleting duplicate characters.
    // I.e.:
    // abc -> abbbc
    // abbbc -> abc
    //HomoPolymerInsert,
    //HomoPolymerDelete,
}

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
    pub layers: [AffineLayerCosts; N],
}

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
}

pub fn affine(sub: Cost, open: Cost, extend: Cost) -> AffineCost<2> {
    AffineCost {
        sub: Some(sub),
        ins: None,
        del: None,
        layers: [
            AffineLayerCosts {
                affine_type: AffineLayerType::Insert,
                open,
                extend,
            },
            AffineLayerCosts {
                affine_type: AffineLayerType::Delete,
                open,
                extend,
            },
        ],
    }
}
pub fn affine2(
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
        layers: [
            AffineLayerCosts {
                affine_type: AffineLayerType::Insert,
                open: ins_open,
                extend: ins_extend,
            },
            AffineLayerCosts {
                affine_type: AffineLayerType::Delete,
                open: del_open,
                extend: del_extend,
            },
        ],
    }
}
