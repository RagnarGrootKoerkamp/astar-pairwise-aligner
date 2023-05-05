//! This module contains the `AffineCost` and `LinearCost` cost models.

use pa_types::*;
use std::cmp::{max, min};

/// An affine layer can either correspond to an insertion or deletion.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AffineLayerType {
    InsertLayer,
    DeleteLayer,
}

impl AffineLayerType {
    pub fn base(&self) -> AffineLayerType {
        match self {
            InsertLayer => InsertLayer,
            DeleteLayer => DeleteLayer,
        }
    }
    pub fn is_insert(&self) -> bool {
        match self {
            InsertLayer => true,
            DeleteLayer => false,
        }
    }
    pub fn is_delete(&self) -> bool {
        match self {
            InsertLayer => false,
            DeleteLayer => true,
        }
    }
}

pub use AffineLayerType::*;

/// An affine layer depends on its type, the open cost, and the extend cost.
#[derive(Clone, Debug, Copy, PartialEq)]
pub struct AffineLayerCosts {
    pub affine_type: AffineLayerType,
    pub open: Cost,
    pub extend: Cost,
}

/// A full cost model consists of linear substitution/insertion/delete costs,
/// and zero or more (N) affine layers.
// The constructure is private to this module.
#[non_exhaustive]
#[derive(Clone, Debug, Copy, PartialEq)]
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

impl From<CostModel> for AffineCost<0> {
    fn from(CostModel { sub, open, extend }: CostModel) -> Self {
        assert_eq!(open, 0, "AffineCost<0> cannot handle affine cost models.");
        AffineCost::new(Some(sub), Some(extend), Some(extend), [])
    }
}

impl From<CostModel> for AffineCost<2> {
    fn from(CostModel { sub, open, extend }: CostModel) -> Self {
        assert!(
            open != 0,
            "AffineCost<2> can only handle affine cost models."
        );
        AffineCost::new(
            Some(sub),
            None,
            None,
            [
                AffineLayerCosts {
                    affine_type: AffineLayerType::InsertLayer,
                    open,
                    extend,
                },
                AffineLayerCosts {
                    affine_type: AffineLayerType::DeleteLayer,
                    open,
                    extend,
                },
            ],
        )
    }
}

impl AffineCost<0> {
    pub fn lcs() -> AffineCost<0> {
        Self::new(None, Some(1), Some(1), [])
    }
    pub fn unit() -> AffineCost<0> {
        Self::new(Some(1), Some(1), Some(1), [])
    }
    pub fn linear(sub: Cost, indel: Cost) -> AffineCost<0> {
        Self::new(Some(sub), Some(indel), Some(indel), [])
    }
    pub fn linear_asymmetric(sub: Cost, ins: Cost, del: Cost) -> AffineCost<0> {
        Self::new(Some(sub), Some(ins), Some(del), [])
    }
}

impl AffineCost<2> {
    pub fn affine(sub: Cost, open: Cost, extend: Cost) -> AffineCost<2> {
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
    pub fn linear_affine(sub: Cost, indel: Cost, open: Cost, extend: Cost) -> AffineCost<2> {
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
    pub fn affine_asymmetric(
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
    pub fn double_affine(
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
        assert!(sub.unwrap_or(1) > 0);
        assert!(ins.unwrap_or(1) > 0);
        assert!(del.unwrap_or(1) > 0);
        for layer in &affine {
            assert!(layer.open > 0);
            assert!(layer.extend > 0);
        }

        let layers = |affine_type| {
            affine
                .iter()
                .filter(move |cm| cm.affine_type.base() == affine_type)
        };
        let min_by = |affine_type, f: &dyn Fn(&AffineLayerCosts) -> Cost| {
            let mut c = layers(affine_type).map(f).min().unwrap_or(Cost::MAX);
            // Also include the linear layer in the affine maximums.
            if let Some(extend) = if affine_type.is_insert() { ins } else { del } {
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
            if let Some(extend) = if affine_type.is_insert() { ins } else { del } {
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

    #[inline]
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

    #[inline]
    pub fn sub_or<U, F>(&self, default: U, f: F) -> U
    where
        F: FnOnce(Cost) -> U,
    {
        self.sub.map_or(default, f)
    }

    #[inline]
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

    #[inline]
    pub fn ins_or<U, F>(&self, default: U, f: F) -> U
    where
        F: FnOnce(Cost) -> U,
    {
        self.ins.map_or(default, f)
    }

    #[inline]
    pub fn del_or<U, F>(&self, default: U, f: F) -> U
    where
        F: FnOnce(Cost) -> U,
    {
        self.del.map_or(default, f)
    }

    /// NOTE: This also includes the linear insert cost.
    #[inline]
    pub fn for_ins(&self, mut f: impl FnMut(Cost, Cost)) {
        if let Some(ins) = self.ins {
            f(0, ins);
        }
        for cm in &self.affine {
            if cm.affine_type.is_insert() {
                f(cm.open, cm.extend);
            }
        }
    }

    /// NOTE: This also includes the linear delete cost.
    #[inline]
    pub fn for_del(&self, mut f: impl FnMut(Cost, Cost)) {
        if let Some(del) = self.del {
            f(0, del);
        }
        for cm in &self.affine {
            if cm.affine_type.is_delete() {
                f(cm.open, cm.extend);
            }
        }
    }

    /// Returns 0 when insertions are not possible.
    #[inline]
    pub fn max_ins_for_cost(&self, s: Cost) -> I {
        let mut d = 0;
        self.for_ins(|o, e| d = max(d, s.saturating_sub(o) / e));
        d
    }

    /// Returns 0 when deletions are not possible.
    #[inline]
    pub fn max_del_for_cost(&self, s: Cost) -> I {
        let mut d = 0;
        self.for_del(|o, e| d = max(d, s.saturating_sub(o) / e));
        d
    }

    /// The maximum number of inserted characters, where entering an affine layer has cost o.
    #[inline]
    pub fn max_ins_for_cost_open_only(&self, s: Cost) -> I {
        let mut d = 0;
        if let Some(ins) = self.ins {
            d = max(d, s / ins);
        }
        for cm in &self.affine {
            if cm.affine_type.is_insert() && s >= min(cm.open, cm.extend) {
                d = max(d, 1 + (s - min(cm.open, cm.extend)) / cm.extend);
            }
        }
        d
    }

    /// The maximum number of deleted characters, where entering an affine layer has cost o.
    #[inline]
    pub fn max_del_for_cost_open_only(&self, s: Cost) -> I {
        let mut d = 0;
        if let Some(del) = self.del {
            d = max(d, s / del);
        }
        for cm in &self.affine {
            if cm.affine_type.is_delete() && s >= min(cm.open, cm.extend) {
                d = max(d, 1 + (s - min(cm.open, cm.extend)) / cm.extend);
            }
        }
        d
    }

    /// d<0: insertion cost
    /// d=0: substitution cost
    /// d>0: deletion cost
    #[inline]
    pub fn linear_cost_in_direction(&self, d: i32) -> Option<Cost> {
        match d {
            d if d < 0 => self.ins,
            d if d == 0 => self.sub,
            d if d > 0 => self.del,
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn to_cigar(&self, layer: usize) -> CigarOp {
        match self.affine[layer].affine_type {
            InsertLayer => CigarOp::Ins,
            DeleteLayer => CigarOp::Del,
        }
    }

    #[inline]
    pub fn gap_cost(&self, s: Pos, t: Pos) -> Cost {
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
                    if cm.affine_type.is_insert() {
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
                    if cm.affine_type.is_delete() {
                        c = min(c, cm.open + d * cm.extend);
                    }
                }
                assert!(c != Cost::MAX);
                c
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn extend_cost(&self, s: Pos, t: Pos) -> Cost {
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
                    if cm.affine_type.is_insert() {
                        c = min(c, d * cm.extend);
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
                    if cm.affine_type.is_delete() {
                        c = min(c, d * cm.extend);
                    }
                }
                assert!(c != Cost::MAX);
                c
            }
            _ => unreachable!(),
        }
    }
}
