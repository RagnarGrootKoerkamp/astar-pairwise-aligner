use super::*;
use pa_bitpacking::V;

/// A block stores vertical differences at its right edge `i` for rows in `j_range`.
///
#[derive(derivative::Derivative)]
#[derivative(Clone(clone_from = "true"))]
pub struct Block {
    /// The vertical differences at the end of block.
    pub v: Vec<V>,
    /// The column of this block.
    pub i_range: IRange,
    /// The range of rows to be computed.
    pub original_j_range: JRange,
    /// The rounded-out range of rows to be computed.
    pub j_range: RoundedOutJRange,
    /// The range of rows with `f(u) <= f_max`.
    /// Always rounded in (we underestimate fixed cells).
    pub fixed_j_range: Option<JRange>,

    /// The `j` of the first element of `v`.
    /// Can be different from `j_range.0` when only a slice of the array corresponds to the `j_range`.
    pub offset: I,
    /// The value at the top of the rounded range, set on construction.
    pub top_val: Cost,
    /// The value at the bottom of the rounded range, computed after the range itself.
    pub bot_val: Cost,

    /// Horizontal differences for row `j_h`, which will be the end of the fixed j_range if set.
    pub j_h: Option<I>,
}

impl Default for Block {
    fn default() -> Self {
        Self {
            v: vec![],
            i_range: IRange(-1, 0),
            original_j_range: JRange(-WI, -WI),
            j_range: JRange(-WI, -WI).round_out(),
            fixed_j_range: None,
            offset: 0,
            top_val: Cost::MAX,
            bot_val: Cost::MAX,
            j_h: None,
        }
    }
}

impl Block {
    /// The initial block for the first column.
    pub fn first_col(original_j_range: JRange, j_range: RoundedOutJRange) -> Self {
        assert!(j_range.0 == 0);
        Self {
            v: vec![V::one(); j_range.exclusive_len() as usize / W],
            i_range: IRange(-1, 0),
            original_j_range,
            j_range,
            // In the first col, all computed values are correct directly.
            fixed_j_range: Some(original_j_range),
            offset: 0,
            top_val: 0,
            bot_val: j_range.exclusive_len(),
            j_h: None,
        }
    }

    /// Get the value at the given index, by counting bits from the top or bottom.
    /// For `j` larger than the range, vertical deltas of `1` are assumed.
    pub fn index(&self, j: I) -> Cost {
        let j_range = self.j_range;
        assert!(
            j_range.0 <= j,
            "Cannot index block {:?} with range {:?} by {}",
            self.i_range,
            j_range,
            j
        );
        // All of rounded must be indexable.
        assert!(
            j_range.0 - self.offset >= 0,
            "Offset too large: {} - {} = {}, jrange {:?}",
            j_range.0,
            self.offset,
            j_range.0 - self.offset,
            self.j_range
        );
        assert!(
            j_range.1 - self.offset <= self.v.len() as I * WI,
            "v not long enough: {} - {} = {}, v len {}, jrange {:?}",
            j_range.1,
            self.offset,
            j_range.1 - self.offset,
            self.v.len() * W,
            self.j_range
        );

        if j > j_range.1 {
            return self.bot_val + (j - j_range.1) as Cost;
        }
        if j - j_range.0 < j_range.1 - j {
            // go from top
            let mut val = self.top_val;
            let mut j0 = j_range.0;
            while j0 + WI <= j {
                val += self.v[(j0 - self.offset) as usize / W].value() as Cost;
                j0 += WI;
            }
            val + self.v[(j0 - self.offset) as usize / W].value_of_prefix(j - j0) as Cost
        } else {
            // go from bottom
            let mut val = self.bot_val;
            let mut j1 = j_range.1;
            while j1 - WI > j {
                val -= self.v[(j1 - WI - self.offset) as usize / W].value() as Cost;
                j1 -= WI;
            }
            if j1 > j {
                val -= self.v[(j1 - WI - self.offset) as usize / W].value_of_suffix(j1 - j) as Cost
            }
            val
        }
    }

    /// Get the value at the given index, by counting bits from the top or bottom.
    /// For `j` outside the range, `None` is returned.
    pub fn get(&self, j: I) -> Option<Cost> {
        if j < self.j_range.0 || j > self.j_range.1 {
            return None;
        }
        Some(self.index(j))
    }

    /// Get the difference from row `j` to `j+1`.
    pub fn get_diff(&self, j: I) -> Option<Cost> {
        if j < self.offset {
            return None;
        }
        let idx = (j - self.offset) as usize / W;
        if idx >= self.v.len() {
            return None;
        }
        let bit = (j - self.offset) as usize % W;

        Some(((self.v[idx].p() >> bit) & 1) as Cost - ((self.v[idx].m() >> bit) & 1) as Cost)
    }

    /// Assert that the vertical difference between the top and bottom values is correct.
    pub fn check_top_bot_val(&self) {
        if !DEBUG {
            return;
        }
        let mut val = self.top_val;
        for v in &self.v[(self.j_range.0 - self.offset) as usize / W
            ..(self.j_range.1 - self.offset) as usize / W]
        {
            val += v.value();
        }
        assert_eq!(val, self.bot_val);
    }
}
