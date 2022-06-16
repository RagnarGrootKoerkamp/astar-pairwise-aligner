use std::fmt::Write;

#[derive(PartialEq)]
pub enum CigarOp {
    Match,
    Mismatch,
    Insertion,
    Deletion,
}

impl CigarOp {
    fn get_char(&self) -> char {
        match self {
            CigarOp::Match => 'M',
            CigarOp::Mismatch => 'X',
            CigarOp::Insertion => 'I',
            CigarOp::Deletion => 'D',
        }
    }

    fn new(self) -> CigarElement {
        CigarElement {
            command: self,
            length: 1,
        }
    }
}

pub struct CigarElement {
    command: CigarOp,
    length: usize,
}

pub type Cigar = Vec<CigarElement>;

pub trait CigarTrait {
    fn cigar_push(&mut self, command: CigarOp);
    fn print(&self);
    fn str(&self) -> String;
}
impl CigarTrait for Cigar {
    fn cigar_push(&mut self, command: CigarOp) {
        if let Some(s) = self.last_mut() {
            if s.command == command {
                s.length += 1;
                return;
            }
        }
        self.push(command.new());
    }

    fn print(&self) {
        for i in self {
            print!("{}{}", i.length, i.command.get_char());
        }
    }

    fn str(&self) -> String {
        let mut str = String::new();
        for i in self {
            write!(&mut str, "{}{}", i.length, i.command.get_char())
                .expect("Error! Unable to write to the string in CIGAR str function!");
        }
        str
    }
}
