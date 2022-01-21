/// A potentially smaller cost datatype to save stack space.
pub type MatchCost = u8;

/// TODO: Gap-Affine costs.
pub struct CostModel {
    pub mismatch: MatchCost,
    pub insertion: MatchCost,
    pub deletion: MatchCost,
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
    mismatch: MatchCost::MAX,
    insertion: 1,
    deletion: 1,
};
