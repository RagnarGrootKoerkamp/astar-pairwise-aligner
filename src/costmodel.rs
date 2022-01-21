use crate::prelude::Cost;

/// TODO: Gap-Affine costs.
pub struct CostModel {
    pub mismatch: Cost,
    pub insertion: Cost,
    pub deletion: Cost,
}

/// Default costs for EditDistance:
/// mismatch, insertion, and deletion all cost 1.
pub const EDIT_DISTANCE_COSTS: CostModel = CostModel {
    mismatch: 1,
    insertion: 1,
    deletion: 1,
};

/// LCS corresponds to disallowing mismatches.
pub const LCS_COSTS: CostModel = CostModel {
    mismatch: Cost::MAX,
    insertion: 1,
    deletion: 1,
};
