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
    fn ins(&self) -> Cost;

    /// The cost of deleting a character, or extending a deletion.
    fn del(&self) -> Cost;

    /// The cost of opening an insertion.
    fn ins_open(&self) -> Cost {
        0
    }

    /// The cost of opening a deletion.
    fn del_open(&self) -> Cost {
        0
    }
}

/// Implement this trait to indicate that the cost model does not use affine costs.
pub trait LinearCostModel: CostModel {}

pub struct LinearCost {
    /// The substitution cost. None for LCS where substitutions are not allowed.
    sub: Option<Cost>,
    ins: Cost,
    del: Cost,
}

impl CostModel for LinearCost {
    fn sub(&self) -> Option<Cost> {
        self.sub
    }

    fn ins(&self) -> Cost {
        self.ins
    }

    fn del(&self) -> Cost {
        self.del
    }
}
impl LinearCostModel for LinearCost {}

impl LinearCost {
    pub fn lcs() -> Self {
        Self {
            sub: None,
            ins: 1,
            del: 1,
        }
    }
    pub fn unit() -> Self {
        Self {
            sub: Some(1),
            ins: 1,
            del: 1,
        }
    }
    pub fn edit_cost(sub: Cost, indel: Cost) -> Self {
        Self {
            sub: Some(sub),
            ins: indel,
            del: indel,
        }
    }
    pub fn edit_cost2(sub: Cost, ins: Cost, del: Cost) -> Self {
        Self {
            sub: Some(sub),
            ins,
            del,
        }
    }

    pub fn to_affine(&self) -> AffineCost {
        AffineCost {
            sub: self.sub,
            ins: self.ins,
            del: self.del,
            ins_open: 0,
            del_open: 0,
        }
    }
}

pub struct AffineCost {
    sub: Option<Cost>,
    ins: Cost,
    del: Cost,
    ins_open: Cost,
    del_open: Cost,
}

impl CostModel for AffineCost {
    fn sub(&self) -> Option<Cost> {
        self.sub
    }

    fn ins(&self) -> Cost {
        self.ins
    }

    fn del(&self) -> Cost {
        self.del
    }

    fn ins_open(&self) -> Cost {
        self.ins_open
    }

    fn del_open(&self) -> Cost {
        self.del_open
    }
}

impl AffineCost {
    pub fn affine(sub: Cost, open: Cost, extend: Cost) -> Self {
        Self {
            sub: Some(sub),
            ins: extend,
            del: extend,
            ins_open: open,
            del_open: open,
        }
    }
    pub fn affine2(sub: Cost, ins_open: Cost, ins: Cost, del_open: Cost, del: Cost) -> Self {
        Self {
            sub: Some(sub),
            ins,
            del,
            ins_open,
            del_open,
        }
    }
}
