use super::{cigar::Cigar, Aligner, Seq};

use crate::{aligners::cigar::CigarOp, cost_model::Cost};
use edlib_rs::{edlibAlignRs, EdlibAlignConfigRs, EdlibAlignTaskRs, EDLIB_RS_STATUS_OK};

#[derive(Debug)]
pub struct Edlib;

fn edlib_align(a: Seq, b: Seq, trace: bool, f_max: Option<Cost>) -> Option<(Cost, Option<Cigar>)> {
    let mut config = EdlibAlignConfigRs::default();
    if trace {
        config.task = EdlibAlignTaskRs::EDLIB_TASK_PATH;
    }
    if let Some(f_max) = f_max {
        config.k = f_max as i32;
    }
    let result = edlibAlignRs(a, b, &config);
    assert!(result.status == EDLIB_RS_STATUS_OK);

    if result.editDistance == -1 {
        return None;
    }

    let distance = result.editDistance as Cost;
    let cigar = result.alignment.map(|alignment| {
        let mut cigar = Cigar::default();
        for op in alignment {
            cigar.push(match op {
                0 => CigarOp::Match,
                1 => CigarOp::Del,
                2 => CigarOp::Ins,
                3 => CigarOp::Sub,
                _ => panic!(),
            });
        }
        cigar
    });
    Some((distance, cigar))
}

impl Aligner for Edlib {
    fn cost(&mut self, a: Seq, b: Seq) -> Cost {
        edlib_align(a, b, false, None).unwrap().0
    }

    fn align(&mut self, a: Seq, b: Seq) -> (Cost, Cigar) {
        let (d, c) = edlib_align(a, b, false, None).unwrap();
        (d, c.unwrap())
    }

    fn cost_for_bounded_dist(&mut self, a: Seq, b: Seq, f_max: Cost) -> Option<Cost> {
        Some(edlib_align(a, b, false, Some(f_max))?.0)
    }

    fn align_for_bounded_dist(&mut self, a: Seq, b: Seq, f_max: Cost) -> Option<(Cost, Cigar)> {
        let (d, c) = edlib_align(a, b, false, Some(f_max))?;
        Some((d, c.unwrap()))
    }
}
