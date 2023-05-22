use crate::cost_model::{AffineCost, AffineLayerType};
use pa_types::*;
use std::slice;

/// A CigarOp with extra markers for affine indel layers.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AffineCigarOp {
    Match,
    Sub,
    /// Linear cost insertion.
    Ins,
    /// Linear cost deletion.
    Del,
    /// Affine cost insertion in given layer.
    AffineIns(usize),
    /// Affine cost deletion in given layer.
    AffineDel(usize),
    // Extra markers that do not translate to commands.
    /// Set when entering an affine layer.
    AffineOpen(usize),
    /// Set when leaving an affine layer.
    AffineClose(usize),
}

#[derive(Debug, Eq, PartialEq)]
pub struct AffineCigarElem {
    pub op: AffineCigarOp,
    pub cnt: I,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct AffineCigar {
    ops: Vec<AffineCigarElem>,
}

impl ToString for AffineCigar {
    fn to_string(&self) -> String {
        self.to_base().to_string()
    }
}

impl AffineCigarOp {
    pub fn to_base(&self) -> Option<CigarOp> {
        Some(match self {
            AffineCigarOp::Match => CigarOp::Match,
            AffineCigarOp::Sub => CigarOp::Sub,
            AffineCigarOp::Ins => CigarOp::Ins,
            AffineCigarOp::Del => CigarOp::Del,
            AffineCigarOp::AffineIns(_) => CigarOp::Ins,
            AffineCigarOp::AffineDel(_) => CigarOp::Del,
            AffineCigarOp::AffineOpen(_) => return None,
            AffineCigarOp::AffineClose(_) => return None,
        })
    }
}

impl From<CigarOp> for AffineCigarOp {
    fn from(op: CigarOp) -> Self {
        match op {
            CigarOp::Match => AffineCigarOp::Match,
            CigarOp::Sub => AffineCigarOp::Sub,
            CigarOp::Del => AffineCigarOp::Del,
            CigarOp::Ins => AffineCigarOp::Ins,
        }
    }
}

impl From<&Cigar> for AffineCigar {
    fn from(cigar: &Cigar) -> Self {
        Self {
            ops: cigar
                .ops
                .iter()
                .map(|el| AffineCigarElem {
                    op: el.op.into(),
                    cnt: el.cnt,
                })
                .collect(),
        }
    }
}

impl Into<CigarOp> for AffineCigarOp {
    fn into(self) -> CigarOp {
        match self {
            AffineCigarOp::Match => CigarOp::Match,
            AffineCigarOp::Sub => CigarOp::Sub,
            AffineCigarOp::Ins => CigarOp::Ins,
            AffineCigarOp::Del => CigarOp::Del,
            _ => panic!("Can not convert affine operations into CigarOp base."),
        }
    }
}

impl Into<Cigar> for AffineCigar {
    fn into(self) -> Cigar {
        Cigar {
            ops: self
                .ops
                .iter()
                .map(|el| CigarElem {
                    op: el.op.into(),
                    cnt: el.cnt,
                })
                .collect(),
        }
    }
}

impl AffineCigar {
    pub fn to_base(&self) -> Cigar {
        Cigar {
            ops: self
                .ops
                .iter()
                .filter_map(|elem| {
                    Some(CigarElem {
                        op: elem.op.to_base()?,
                        cnt: elem.cnt,
                    })
                })
                .collect(),
        }
    }

    pub fn push_op(&mut self, op: AffineCigarOp) {
        // TODO: Make sure that Affine{Insert,Delete} can only come after an Open/Close.
        if let Some(s) = self.ops.last_mut() {
            if s.op == op {
                s.cnt += 1;
                return;
            }
        }
        self.ops.push(AffineCigarElem { op, cnt: 1 });
    }

    pub fn push_elem(&mut self, elem: AffineCigarElem) {
        // TODO: Make sure that Affine{Insert,Delete} can only come after an Open/Close.
        if let Some(s) = self.ops.last_mut() {
            if s.op == elem.op {
                s.cnt += elem.cnt;
                return;
            }
        }
        self.ops.push(elem);
    }

    /// Extend the cigar by the given number of matches.
    pub fn match_push(&mut self, cnt: I) {
        if let Some(s) = self.ops.last_mut() {
            if s.op == AffineCigarOp::Match {
                s.cnt += cnt;
                return;
            }
        }
        self.ops.push(AffineCigarElem {
            op: AffineCigarOp::Match,
            cnt,
        });
    }

    /// Reverse the cigar string.
    pub fn reverse(&mut self) {
        self.ops.reverse()
    }

    /// Append another cigar to this one.
    pub fn append(&mut self, other: &mut Self) {
        let Some(first) = other.ops.first_mut() else {return;};
        if let Some(s) = self.ops.last_mut() {
            if s.op == first.op {
                first.cnt += s.cnt;
                self.ops.pop().unwrap();
            }
        }
        self.ops.append(&mut other.ops);
    }

    pub fn to_path(&self) -> Path {
        self.to_base().to_path()
    }

    pub fn to_path_with_costs<const N: usize>(&self, cm: AffineCost<N>) -> Vec<(Pos, Cost)> {
        let mut pos = Pos(0, 0);
        let mut layer = None;
        let mut cost = 0;
        let mut path = vec![(pos, cost)];

        for el in &self.ops {
            let length = el.cnt as Cost;
            match el.op {
                AffineCigarOp::Match => {
                    assert!(layer == None);
                    for _ in 0..length {
                        pos.0 += 1;
                        pos.1 += 1;
                        path.push((pos, cost));
                    }
                }
                AffineCigarOp::Sub => {
                    assert!(layer == None);
                    for _ in 0..length {
                        pos.0 += 1;
                        pos.1 += 1;
                        cost += cm.sub.unwrap();
                        path.push((pos, cost));
                    }
                }
                AffineCigarOp::Ins => {
                    assert!(layer == None);
                    for _ in 0..length {
                        pos.1 += 1;
                        cost += cm.ins.unwrap();
                        path.push((pos, cost));
                    }
                }
                AffineCigarOp::Del => {
                    assert!(layer == None);
                    for _ in 0..length {
                        pos.0 += 1;
                        cost += cm.del.unwrap();
                        path.push((pos, cost));
                    }
                }
                AffineCigarOp::AffineIns(l) => {
                    assert_eq!(layer, Some(l));
                    assert_eq!(
                        cm.affine[l].affine_type.base(),
                        AffineLayerType::InsertLayer
                    );
                    for _ in 0..length {
                        pos.1 += 1;
                        cost += cm.affine[l].extend;
                        path.push((pos, cost));
                    }
                }
                AffineCigarOp::AffineDel(l) => {
                    assert_eq!(layer, Some(l));
                    assert_eq!(
                        cm.affine[l].affine_type.base(),
                        AffineLayerType::DeleteLayer
                    );
                    for _ in 0..length {
                        pos.0 += 1;
                        cost += cm.affine[l].extend;
                        path.push((pos, cost));
                    }
                }
                AffineCigarOp::AffineOpen(l) => {
                    assert_eq!(layer, None);
                    cost += cm.affine[l].open;
                    layer = Some(l)
                }
                AffineCigarOp::AffineClose(l) => {
                    assert_eq!(layer, Some(l));
                    layer = None;
                }
            }
        }
        path
    }

    pub fn verify<const N: usize>(&self, cm: &AffineCost<N>, a: Seq, b: Seq) -> Cost {
        let mut pos = Pos(0, 0);
        let mut layer = None;
        let mut cost = 0;

        for &AffineCigarElem {
            op: command,
            cnt: length,
        } in self
        {
            match command {
                AffineCigarOp::Match => {
                    assert!(layer == None);
                    for _ in 0..length {
                        assert_eq!(a.get(pos.0 as usize), b.get(pos.1 as usize));
                        pos.0 += 1;
                        pos.1 += 1;
                    }
                }
                AffineCigarOp::Sub => {
                    assert!(layer == None);
                    for _ in 0..length {
                        assert_ne!(a.get(pos.0 as usize), b.get(pos.1 as usize));
                        pos.0 += 1;
                        pos.1 += 1;
                        cost += cm.sub.unwrap();
                    }
                }
                AffineCigarOp::Ins => {
                    assert!(layer == None);
                    pos.1 += length;
                    cost += cm.ins.unwrap() * length as Cost;
                }
                AffineCigarOp::Del => {
                    assert!(layer == None);
                    pos.0 += length;
                    cost += cm.del.unwrap() * length as Cost;
                }
                AffineCigarOp::AffineIns(l) => {
                    assert_eq!(layer, Some(l));
                    assert_eq!(
                        cm.affine[l].affine_type.base(),
                        AffineLayerType::InsertLayer
                    );
                    pos.1 += length;
                    cost += cm.affine[l].extend * length as Cost;
                }
                AffineCigarOp::AffineDel(l) => {
                    assert_eq!(layer, Some(l));
                    assert_eq!(
                        cm.affine[l].affine_type.base(),
                        AffineLayerType::DeleteLayer
                    );
                    pos.0 += length;
                    cost += cm.affine[l].extend * length as Cost;
                }
                AffineCigarOp::AffineOpen(l) => {
                    assert_eq!(layer, None);
                    cost += cm.affine[l].open;
                    layer = Some(l)
                }
                AffineCigarOp::AffineClose(l) => {
                    assert_eq!(layer, Some(l));
                    layer = None;
                }
            }
        }

        cost
    }
}

impl<'a> IntoIterator for &'a AffineCigar {
    type Item = &'a AffineCigarElem;

    type IntoIter = slice::Iter<'a, AffineCigarElem>;

    fn into_iter(self) -> Self::IntoIter {
        self.ops.iter()
    }
}
