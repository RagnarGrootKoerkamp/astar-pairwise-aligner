use std::{fmt::Write, slice};

use itertools::Itertools;

use crate::prelude::Pos;

use super::{Path, Seq};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum CigarOp {
    Match,
    Mismatch,
    /// Linear cost insertion.
    Insertion,
    /// Linear cost deletion.
    Deletion,
    /// Affine cost insertion in given layer.
    AffineInsertion(usize),
    /// Affine cost deletion in given layer.
    AffineDeletion(usize),
    // Extra markers that do not translate to commands.
    /// Set when entering an affine layer.
    AffineOpen(usize),
    /// Set when leaving an affine layer.
    AffineClose(usize),
}

impl CigarOp {
    /// Not all operations have an actual cigar character.
    fn get_char(&self) -> Option<char> {
        match self {
            CigarOp::Match => Some('M'),
            CigarOp::Mismatch => Some('X'),
            CigarOp::Insertion => Some('I'),
            CigarOp::Deletion => Some('D'),
            CigarOp::AffineInsertion(_) => Some('I'),
            CigarOp::AffineDeletion(_) => Some('D'),
            _ => None,
        }
    }

    fn delta(&self) -> Option<Pos> {
        match self {
            CigarOp::Match => Some(Pos(1, 1)),
            CigarOp::Mismatch => Some(Pos(1, 1)),
            CigarOp::Insertion | CigarOp::AffineInsertion(_) => Some(Pos(0, 1)),
            CigarOp::Deletion | CigarOp::AffineDeletion(_) => Some(Pos(1, 0)),
            _ => None,
        }
    }

    fn new(self) -> CigarElement {
        let length = match &self {
            CigarOp::AffineOpen(_) | CigarOp::AffineClose(_) => 0,
            _ => 1,
        };
        CigarElement {
            command: self,
            length,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct CigarElement {
    pub command: CigarOp,
    pub length: usize,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct Cigar {
    ops: Vec<CigarElement>,
}

impl ToString for Cigar {
    fn to_string(&self) -> String {
        let mut s = String::new();
        for op in &self.ops {
            if let Some(c) = op.command.get_char() {
                write!(&mut s, "{}{}", op.length, c).unwrap();
            }
        }
        s
    }
}

impl Cigar {
    fn match_pos(delta: Pos, pos: Pos, a: Seq, b: Seq) -> CigarOp {
        match delta {
            Pos(1, 1) => {
                assert!(pos.0 > 0 && pos.1 > 0);
                if a[(pos.0 - 1) as usize] == b[(pos.1 - 1) as usize] {
                    CigarOp::Match
                } else {
                    CigarOp::Mismatch
                }
            }
            Pos(0, 1) => CigarOp::Insertion,
            Pos(1, 0) => CigarOp::Deletion,
            _ => panic!("Offset is not correct"),
        }
    }

    pub fn from_path(a: Seq, b: Seq, path: &Path) -> Cigar {
        if path[0] != Pos(0, 0) {
            panic!("Path must start at (0,0)!");
        }
        let mut cigar = Cigar::default();
        for (&prev, &cur) in path.iter().tuple_windows() {
            cigar.push(Self::match_pos(cur - prev, cur, a, b));
        }
        cigar
    }

    pub fn push(&mut self, command: CigarOp) {
        // TODO: Make sure that Affine{Insert,Delete} can only come after an Open/Close.
        if let Some(s) = self.ops.last_mut() && s.command == command {
            s.length += 1;
            return;
        }
        self.ops.push(command.new());
    }

    pub fn match_push(&mut self, num: usize) {
        if let Some(s) = self.ops.last_mut() {
            if s.command == CigarOp::Match {
                s.length += num;
                return;
            }
        }
        self.ops.push(CigarElement {
            command: CigarOp::Match,
            length: num,
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
        if let Some(s) = self.ops.last_mut() && s.command == first.command{
            first.length += s.length;
            self.ops.pop().unwrap();
        }
        self.ops.append(&mut other.ops);
    }

    pub fn to_path(&self) -> Path {
        let mut position = Pos(0, 0);
        let mut path = vec![position];
        for el in &self.ops {
            for _ in 0..el.length {
                if let Some(offset) = el.command.delta() {
                    position = position + offset;
                    path.push(position);
                }
            }
        }
        path
    }
}

impl<'a> IntoIterator for &'a Cigar {
    type Item = &'a CigarElement;

    type IntoIter = slice::Iter<'a, CigarElement>;

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

        for &CigarElement { command, length } in cigar {
            match command {
                CigarOp::Match => {
                    assert!(layer == None);
                    for _ in 0..length {
                        assert_eq!(a.get(pos.0), b.get(pos.1));
                        pos.0 += 1;
                        pos.1 += 1;
                    }
                }
                CigarOp::Mismatch => {
                    assert!(layer == None);
                    for _ in 0..length {
                        assert_ne!(a.get(pos.0), b.get(pos.1));
                        pos.0 += 1;
                        pos.1 += 1;
                        cost += cm.sub.unwrap();
                    }
                }
                CigarOp::Insertion => {
                    assert!(layer == None);
                    pos.1 += length;
                    cost += cm.ins.unwrap() * length as Cost;
                }
                CigarOp::Deletion => {
                    assert!(layer == None);
                    pos.0 += length;
                    cost += cm.del.unwrap() * length as Cost;
                }
                CigarOp::AffineInsertion(l) => {
                    assert_eq!(layer, Some(l));
                    assert_eq!(
                        cm.affine[l].affine_type.base(),
                        AffineLayerType::InsertLayer
                    );
                    pos.1 += length;
                    cost += cm.affine[l].extend * length as Cost;
                }
                CigarOp::AffineDeletion(l) => {
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
