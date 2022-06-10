//! This module contains various cost models.

use crate::matches::MatchCost;

/// Implement this trait to indicate that the cost model does not use affine costs.
pub trait LinearCostModel {
    /// The cost of a substitution.
    /// TODO: Make this depend on the characters being substituted.
    /// Note that this returns an optional, because in case of LCS costs substitutions are not allowed.
    fn sub(&self) -> Option<MatchCost>;

    /// The cost of inserting a character, or extending an insert.
    fn ins(&self) -> MatchCost;

    /// The cost of deleting a character, or extending a deletion.
    fn del(&self) -> MatchCost;
}

pub trait AffineCostModel: LinearCostModel {
    /// The cost of opening an insertion.
    fn ins_open(&self) -> MatchCost {
        0
    }

    /// The cost of opening a deletion.
    fn del_open(&self) -> MatchCost {
        0
    }
}

pub struct EditCost {
    /// The substitution cost. None for LCS where substitutions are not allowed.
    sub: Option<MatchCost>,
    ins: MatchCost,
    del: MatchCost,
}

impl LinearCostModel for EditCost {
    fn sub(&self) -> Option<MatchCost> {
        self.sub
    }

    fn ins(&self) -> MatchCost {
        self.ins
    }

    fn del(&self) -> MatchCost {
        self.del
    }
}

impl EditCost {
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
    pub fn edit_cost(sub: MatchCost, indel: MatchCost) -> Self {
        Self {
            sub: Some(sub),
            ins: indel,
            del: indel,
        }
    }
    pub fn edit_cost2(sub: MatchCost, ins: MatchCost, del: MatchCost) -> Self {
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
    sub: Option<MatchCost>,
    ins: MatchCost,
    del: MatchCost,
    ins_open: MatchCost,
    del_open: MatchCost,
}

impl LinearCostModel for AffineCost {
    fn sub(&self) -> Option<MatchCost> {
        self.sub
    }

    fn ins(&self) -> MatchCost {
        self.ins
    }

    fn del(&self) -> MatchCost {
        self.del
    }
}

impl AffineCostModel for AffineCost {
    fn ins_open(&self) -> MatchCost {
        self.ins_open
    }

    fn del_open(&self) -> MatchCost {
        self.del_open
    }
}

impl AffineCost {
    pub fn affine(sub: MatchCost, open: MatchCost, extend: MatchCost) -> Self {
        Self {
            sub: Some(sub),
            ins: extend,
            del: extend,
            ins_open: open,
            del_open: open,
        }
    }
    pub fn affine2(
        sub: MatchCost,
        ins_open: MatchCost,
        ins: MatchCost,
        del_open: MatchCost,
        del: MatchCost,
    ) -> Self {
        Self {
            sub: Some(sub),
            ins,
            del,
            ins_open,
            del_open,
        }
    }
}
