use super::{Aligner, Seq};
use crate::cost_model::{AffineCost, AffineLayerType, Cost};
use rust_wfa2::aligner::{
    AlignmentScope, AlignmentStatus, Heuristic, MemoryModel, WFAligner, WFAlignerEdit,
    WFAlignerGapAffine, WFAlignerGapAffine2Pieces, WFAlignerGapLinear, WFAlignerIndel,
};

pub struct WFA<CostModel> {
    pub cm: CostModel,
}

impl<CostModel> std::fmt::Debug for WFA<CostModel> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WFA").finish()
    }
}

// NOTE: All of the functions below internally compute the full alignment, but only return the score.

fn align(a: Seq, b: Seq, mut aligner: WFAligner) -> i32 {
    aligner.set_heuristic(Heuristic::None);
    let status = aligner.align_end_to_end(a, b);
    assert_eq!(status, AlignmentStatus::StatusSuccessful);
    aligner.score()
}

fn lcs_cost(a: Seq, b: Seq) -> Cost {
    let aligner = WFAlignerIndel::new(AlignmentScope::Alignment, MemoryModel::MemoryUltraLow);
    align(a, b, aligner) as _
}

fn unit_cost(a: Seq, b: Seq) -> Cost {
    let aligner = WFAlignerEdit::new(AlignmentScope::Alignment, MemoryModel::MemoryUltraLow);
    align(a, b, aligner) as _
}

fn linear_cost(a: Seq, b: Seq, sub: Cost, indel: Cost) -> Cost {
    let aligner = WFAlignerGapLinear::new(
        sub as _,
        indel as _,
        AlignmentScope::Alignment,
        MemoryModel::MemoryUltraLow,
    );
    -align(a, b, aligner) as Cost
}

fn affine_cost(a: Seq, b: Seq, sub: Cost, open: Cost, extend: Cost) -> Cost {
    let aligner = WFAlignerGapAffine::new(
        sub as _,
        open as _,
        extend as _,
        AlignmentScope::Alignment,
        MemoryModel::MemoryUltraLow,
    );
    -align(a, b, aligner) as Cost
}

fn double_affine_cost(
    a: Seq,
    b: Seq,
    sub: Cost,
    open1: Cost,
    extend1: Cost,
    open2: Cost,
    extend2: Cost,
) -> Cost {
    let aligner = WFAlignerGapAffine2Pieces::new(
        sub as _,
        open1 as _,
        extend1 as _,
        open2 as _,
        extend2 as _,
        AlignmentScope::Alignment,
        MemoryModel::MemoryUltraLow,
    );
    -align(a, b, aligner) as Cost
}

impl<const N: usize> Aligner for WFA<AffineCost<N>> {
    fn cost(&mut self, a: Seq, b: Seq) -> Cost {
        let mut cost = (|| {
            if N == 0 {
                //lcs cost
                if self.cm.sub == None && self.cm.ins == self.cm.del{
                return lcs_cost(a, b);
                //unit cost
            } else if self.cm.sub == Some(1) && self.cm.ins == Some(1) && self.cm.del == Some(1){
                return unit_cost(a, b);
                //linear cost
            } else if let Some(sub) = self.cm.sub
            && let Some(ins) = self.cm.ins
            && let Some(del) = self.cm.del
            && ins == del {
                return linear_cost(a, b, sub, ins);
            }
                //affine cost
            } else if N == 2 {
                if let Some(sub) = self.cm.sub  && self.cm.ins == None && self.cm.del == None {
                let l0 = &self.cm.affine[0];
                let l1 = &self.cm.affine[1];
                if l0.affine_type == AffineLayerType::InsertLayer
                   && l1.affine_type == AffineLayerType::DeleteLayer
                {
                    return affine_cost(
                        a,
                        b,
                        sub,
                        self.cm.affine[0].open,
                        self.cm.affine[0].extend
                    );
                }
            }
            } else if N == 4 {
                let l0 = &self.cm.affine[0];
                let l1 = &self.cm.affine[1];
                let l2 = &self.cm.affine[2];
                let l3 = &self.cm.affine[3];
                if let Some(sub) = self.cm.sub && self.cm.ins == None && self.cm.del == None {
                if l0.affine_type == AffineLayerType::InsertLayer
                    && l1.affine_type == AffineLayerType::DeleteLayer
                    && l2.affine_type == AffineLayerType::InsertLayer
                    && l3.affine_type == AffineLayerType::DeleteLayer
                    && l0.open == l1.open
                    && l0.extend == l1.extend
                    && l2.open == l3.open
                    && l2.extend == l3.extend
                {
                    return double_affine_cost(
                        a,
                        b,
                        sub,
                        self.cm.affine[0].open,
                        self.cm.affine[0].extend,
                        self.cm.affine[2].open,
                        self.cm.affine[2].extend
                    );
                }
            }
            }
            unimplemented!("Cost model is not of a supported type!");
        })();
        // Work around a BiWFA bug.
        if cost == i32::MIN as u32 {
            cost = 0;
        }
        cost
    }
}

#[cfg(test)]
mod tests {
    use rand::{thread_rng, Rng, SeedableRng};

    use crate::{
        aligners::{
            diagonal_transition::{DiagonalTransition, GapCostHeuristic},
            nw::NW,
            Aligner,
        },
        cost_model::LinearCost,
        generate::setup_sequences,
        heuristic::ZeroCost,
        prelude::to_string,
        visualizer::NoVisualizer,
    };

    use super::WFA;

    #[ignore]
    #[test]
    fn biwfa_fuzz() {
        let cm = LinearCost::new_linear(1, 10);
        let mut nw = NW {
            cm: cm.clone(),
            use_gap_cost_heuristic: false,
            exponential_search: false,
            local_doubling: false,
            h: ZeroCost,
            v: NoVisualizer,
        };
        let mut biwfa = WFA { cm: cm.clone() };
        let mut dt = DiagonalTransition::new(
            cm.clone(),
            GapCostHeuristic::Disable,
            ZeroCost,
            true,
            NoVisualizer,
        );
        let _seed = thread_rng().gen_range(0..100000);
        let seed = 51244;
        println!("Seed {seed}");
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
        loop {
            let n = rng.gen_range(10..1000);
            let e = rng.gen_range(0.0..1.0);
            let (ref a, ref b) = setup_sequences(n, e);
            let nw_cost = nw.cost(a, b);
            let biwfa_cost = biwfa.cost(a, b);
            let dt_cost = dt.cost(a, b);

            assert_eq!(
                nw_cost,
                biwfa_cost,
                "\nnw:    {nw_cost}\ndt:    {dt_cost}\nbiwfa: {biwfa_cost}\n{n} {e}\nA\n{}\nB\n{}\nseed: {seed}",
                to_string(&a),
                to_string(&b),
            );
        }
    }
}
