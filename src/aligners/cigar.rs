use std::fmt::Write;

#[derive(PartialEq)]
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

pub struct CigarElement {
    command: CigarOp,
    length: usize,
}

#[derive(Default)]
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
    pub fn push(&mut self, command: CigarOp) {
        // TODO: Make sure that Affine{Insert,Delete} can only come after an Open/Close.
        if let Some(s) = self.ops.last_mut() {
            if s.command == command {
                s.length += 1;
                return;
            }
        }
        self.ops.push(command.new());
    }

    pub fn print(&self) {
        print!("{}", self.to_string());
    }

    pub fn reverse(&mut self) {
        self.ops.reverse()
    }
}
