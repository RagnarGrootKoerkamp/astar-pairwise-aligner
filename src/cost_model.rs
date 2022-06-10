//! This module contains various cost models.

use crate::matches::MatchCost;

/// A costmodel tells the cost of various mutation operations.
/// Some are more general than others.
#[derive(Clone, Copy)]
pub enum CostModel {
    /// Cost 1 indel, no substitutions.
    LCSCost,
    /// Cost 1 indel and substitutions.
    UnitCost,
    /// Different cost for substitutions and indels.
    EditCost { sub: MatchCost, indel: MatchCost },
    /// Asymmetric indel costs.
    EditCost2 {
        sub: MatchCost,
        ins: MatchCost,
        del: MatchCost,
    },
    /// Gap open cost.
    Affine {
        sub: MatchCost,
        open: MatchCost,
        extend: MatchCost,
    },
    /// Asymmetric affine costs.
    Affine2 {
        sub: MatchCost,
        ins_open: MatchCost,
        ins_extend: MatchCost,
        del_open: MatchCost,
        del_extend: MatchCost,
    },
}

impl CostModel {
    /// The cost of a substitution.
    /// TODO: Make this depend on the characters being substituted.
    /// Note that this returns an optional, because in case of LCS costs substitutions are not allowed.
    #[inline]
    pub fn sub(&self) -> Option<MatchCost> {
        match *self {
            CostModel::LCSCost => None,
            CostModel::UnitCost => Some(1),
            CostModel::EditCost { sub, .. }
            | CostModel::EditCost2 { sub, .. }
            | CostModel::Affine { sub, .. }
            | CostModel::Affine2 { sub, .. } => Some(sub),
        }
    }

    /// The cost of opening an insertion.
    #[inline]
    pub fn ins_open(&self) -> MatchCost {
        match *self {
            CostModel::LCSCost
            | CostModel::UnitCost
            | CostModel::EditCost { .. }
            | CostModel::EditCost2 { .. } => 0,
            CostModel::Affine { open, .. } => open,
            CostModel::Affine2 { ins_open, .. } => ins_open,
        }
    }

    /// The cost of opening a deletion.
    #[inline]
    pub fn del_open(&self) -> MatchCost {
        match *self {
            CostModel::LCSCost
            | CostModel::UnitCost
            | CostModel::EditCost { .. }
            | CostModel::EditCost2 { .. } => 0,
            CostModel::Affine { open, .. } => open,
            CostModel::Affine2 { del_open, .. } => del_open,
        }
    }

    /// The cost of inserting a character, or extending an insert.
    #[inline]
    pub fn ins(&self) -> MatchCost {
        match *self {
            CostModel::LCSCost | CostModel::UnitCost => 1,
            CostModel::EditCost { indel, .. } => indel,
            CostModel::EditCost2 { ins, .. } => ins,
            CostModel::Affine { extend, .. } => extend,
            CostModel::Affine2 { ins_extend, .. } => ins_extend,
        }
    }

    /// The cost of deleting a character, or extending a deletion.
    #[inline]
    pub fn del(&self) -> MatchCost {
        match *self {
            CostModel::LCSCost | CostModel::UnitCost => 1,
            CostModel::EditCost { indel, .. } => indel,
            CostModel::EditCost2 { del, .. } => del,
            CostModel::Affine { extend, .. } => extend,
            CostModel::Affine2 { del_extend, .. } => del_extend,
        }
    }
}
