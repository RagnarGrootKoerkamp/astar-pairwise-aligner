use std::{fmt::Write, slice};

use crate::cost_model::{AffineCost, AffineLayerType};
use pa_types::*;

/// A cigarop with extra markers for affine indels.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum CigarOpExt {
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
pub struct CigarElemExt {
    pub op: CigarOpExt,
    pub cnt: usize,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct CigarExt {
    ops: Vec<CigarElemExt>,
}

impl CigarOpExt {
    pub fn to_base(&self) -> Option<CigarOp> {
        Some(match self {
            CigarOpExt::Match => CigarOp::Match,
            CigarOpExt::Sub => CigarOp::Sub,
            CigarOpExt::Ins => CigarOp::Ins,
            CigarOpExt::Del => CigarOp::Del,
            CigarOpExt::AffineIns(_) => CigarOp::Ins,
            CigarOpExt::AffineDel(_) => CigarOp::Del,
            CigarOpExt::AffineOpen(_) => return None,
            CigarOpExt::AffineClose(_) => return None,
        })
    }
}

impl CigarExt {
    pub fn to_base(&self) -> Cigar {
        Cigar {
            ops: self.ops.iter().map(|elem| CigarElem {
                op: elem.op.to_cigar(),
                cnt: elem.cnt,
            }),
        }
    }
}

impl ToString for CigarExt {
    fn to_string(&self) -> String {
        self.to_base().to_string()
    }
}

impl CigarExt {
    fn match_pos(delta: Pos, pos: Pos, a: Seq, b: Seq) -> CigarOpExt {
        match delta {
            Pos(1, 1) => {
                assert!(pos.0 > 0 && pos.1 > 0);
                if a[(pos.0 - 1) as usize] == b[(pos.1 - 1) as usize] {
                    CigarOpExt::Match
                } else {
                    CigarOpExt::Sub
                }
            }
            Pos(0, 1) => CigarOpExt::Ins,
            Pos(1, 0) => CigarOpExt::Del,
            _ => panic!("Offset is not correct"),
        }
    }

    pub fn push(&mut self, command: CigarOpExt) {
        // TODO: Make sure that Affine{Insert,Delete} can only come after an Open/Close.
        if let Some(s) = self.ops.last_mut() && s.op == command {
            s.cnt += 1;
            return;
        }
        self.ops.push(command.new());
    }

    pub fn match_push(&mut self, num: usize) {
        if let Some(s) = self.ops.last_mut() {
            if s.op == CigarOpExt::Match {
                s.cnt += num;
                return;
            }
        }
        self.ops.push(CigarElemExt {
            op: CigarOpExt::Match,
            cnt: num,
        });
    }

    pub fn print(&self) {
        println!("CIGAR: {}", self.to_string());
    }

    pub fn reverse(&mut self) {
        self.ops.reverse()
    }

    pub fn append(&mut self, other: &mut Self) {
        let Some(first) = other.ops.first_mut() else {return;};
        if let Some(s) = self.ops.last_mut() && s.op == first.op{
            first.cnt += s.cnt;
            self.ops.pop().unwrap();
        }
        self.ops.append(&mut other.ops);
    }

    pub fn to_path(&self) -> Path {
        let mut position = Pos(0, 0);
        let mut path = vec![position];
        for el in &self.ops {
            for _ in 0..el.cnt {
                if let Some(offset) = el.op.delta() {
                    position = position + offset;
                    path.push(position);
                }
            }
        }
        path
    }

    pub fn to_path_with_cost<const N: usize>(&self, cm: AffineCost<N>) -> Vec<(Pos, Cost)> {
        let mut pos = Pos(0, 0);
        let mut layer = None;
        let mut cost = 0;
        let mut path = vec![(pos, cost)];

        for el in &self.ops {
            let length = el.cnt as Cost;
            match el.op {
                CigarOpExt::Match => {
                    assert!(layer == None);
                    for _ in 0..length {
                        pos.0 += 1;
                        pos.1 += 1;
                        path.push((pos, cost));
                    }
                }
                CigarOpExt::Sub => {
                    assert!(layer == None);
                    for _ in 0..length {
                        pos.0 += 1;
                        pos.1 += 1;
                        cost += cm.sub.unwrap();
                        path.push((pos, cost));
                    }
                }
                CigarOpExt::Ins => {
                    assert!(layer == None);
                    for _ in 0..length {
                        pos.1 += 1;
                        cost += cm.ins.unwrap();
                        path.push((pos, cost));
                    }
                }
                CigarOpExt::Del => {
                    assert!(layer == None);
                    for _ in 0..length {
                        pos.0 += 1;
                        cost += cm.del.unwrap();
                        path.push((pos, cost));
                    }
                }
                CigarOpExt::AffineIns(l) => {
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
                CigarOpExt::AffineDel(l) => {
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
                CigarOpExt::AffineOpen(l) => {
                    assert_eq!(layer, None);
                    cost += cm.affine[l].open;
                    layer = Some(l)
                }
                CigarOpExt::AffineClose(l) => {
                    assert_eq!(layer, Some(l));
                    layer = None;
                }
            }
        }
        path
    }
}

impl<'a> IntoIterator for &'a CigarExt {
    type Item = &'a CigarElemExt;

    type IntoIter = slice::Iter<'a, CigarElemExt>;

    fn into_iter(self) -> Self::IntoIter {
        self.ops.iter()
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::{
        aligners::Seq,
        prelude::{AffineCost, AffineLayerType, Cost},
    };

    pub fn verify_cigar<const N: usize>(cm: &AffineCost<N>, a: Seq, b: Seq, cigar: &Cigar) -> Cost {
        let mut pos = (0, 0);
        let mut layer = None;
        let mut cost = 0;

        for &CigarElemExt {
            op: command,
            cnt: length,
        } in cigar
        {
            match command {
                CigarOp::Match => {
                    assert!(layer == None);
                    for _ in 0..length {
                        assert_eq!(a.get(pos.0), b.get(pos.1));
                        pos.0 += 1;
                        pos.1 += 1;
                    }
                }
                CigarOp::Sub => {
                    assert!(layer == None);
                    for _ in 0..length {
                        assert_ne!(a.get(pos.0), b.get(pos.1));
                        pos.0 += 1;
                        pos.1 += 1;
                        cost += cm.sub.unwrap();
                    }
                }
                CigarOp::Ins => {
                    assert!(layer == None);
                    pos.1 += length;
                    cost += cm.ins.unwrap() * length as Cost;
                }
                CigarOp::Del => {
                    assert!(layer == None);
                    pos.0 += length;
                    cost += cm.del.unwrap() * length as Cost;
                }
                CigarOp::AffineIns(l) => {
                    assert_eq!(layer, Some(l));
                    assert_eq!(
                        cm.affine[l].affine_type.base(),
                        AffineLayerType::InsertLayer
                    );
                    pos.1 += length;
                    cost += cm.affine[l].extend * length as Cost;
                }
                CigarOp::AffineDel(l) => {
                    assert_eq!(layer, Some(l));
                    assert_eq!(
                        cm.affine[l].affine_type.base(),
                        AffineLayerType::DeleteLayer
                    );
                    pos.0 += length;
                    cost += cm.affine[l].extend * length as Cost;
                }
                CigarOp::AffineOpen(l) => {
                    assert_eq!(layer, None);
                    cost += cm.affine[l].open;
                    layer = Some(l)
                }
                CigarOp::AffineClose(l) => {
                    assert_eq!(layer, Some(l));
                    layer = None;
                }
            }
        }

        cost
    }
}
