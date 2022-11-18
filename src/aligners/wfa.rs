use super::{Aligner, Seq};
use crate::cost_model::{AffineCost, AffineLayerType, Cost};
use std::intrinsics::transmute;

#[allow(non_upper_case_globals)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[allow(unused)]
mod wfa {
    include!(concat!(env!("OUT_DIR"), "/bindings_wfa.rs"));
}

pub struct WFA<CostModel> {
    pub cm: CostModel,
    pub biwfa: bool,
}

impl<CostModel> std::fmt::Debug for WFA<CostModel> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WFA").finish()
    }
}

fn lcs_cost(a: Seq, b: Seq, biwfa: bool) -> Cost {
    unsafe {
        // Configure alignment attributes
        let mut attributes = wfa::wavefront_aligner_attr_default;
        attributes.heuristic.strategy = wfa::wf_heuristic_strategy_wf_heuristic_none;
        attributes.distance_metric = wfa::distance_metric_t_indel;
        attributes.alignment_scope = wfa::alignment_scope_t_compute_score;
        if biwfa {
            attributes.memory_mode = wfa::wavefront_memory_t_wavefront_memory_ultralow;
        }
        // Initialize Wavefront Aligner
        let wf_aligner = wfa::wavefront_aligner_new(&mut attributes);
        let a: &[i8] = transmute(a);
        let b: &[i8] = transmute(b);
        let status = wfa::wavefront_align(
            wf_aligner,
            a.as_ptr(),
            a.len() as i32,
            b.as_ptr(),
            b.len() as i32,
        );
        assert_eq!(status, 0);
        let cost = (*wf_aligner).cigar.score as Cost;
        wfa::wavefront_aligner_delete(wf_aligner);
        cost
    }
}

fn unit_cost(a: Seq, b: Seq, biwfa: bool) -> Cost {
    unsafe {
        // Configure alignment attributes
        let mut attributes = wfa::wavefront_aligner_attr_default;
        attributes.heuristic.strategy = wfa::wf_heuristic_strategy_wf_heuristic_none;
        attributes.distance_metric = wfa::distance_metric_t_edit;
        attributes.alignment_scope = wfa::alignment_scope_t_compute_score;
        if biwfa {
            attributes.memory_mode = wfa::wavefront_memory_t_wavefront_memory_ultralow;
        }
        // Initialize Wavefront Aligner
        let wf_aligner = wfa::wavefront_aligner_new(&mut attributes);
        let a: &[i8] = transmute(a);
        let b: &[i8] = transmute(b);
        let status = wfa::wavefront_align(
            wf_aligner,
            a.as_ptr(),
            a.len() as i32,
            b.as_ptr(),
            b.len() as i32,
        );
        assert_eq!(status, 0);
        let cost = (*wf_aligner).cigar.score as Cost;
        wfa::wavefront_aligner_delete(wf_aligner);
        cost
    }
}

fn linear_cost(a: Seq, b: Seq, sub: Cost, indel: Cost, biwfa: bool) -> Cost {
    unsafe {
        // Configure alignment attributes
        let mut attributes = wfa::wavefront_aligner_attr_default;
        attributes.heuristic.strategy = wfa::wf_heuristic_strategy_wf_heuristic_none;
        attributes.distance_metric = wfa::distance_metric_t_gap_linear;
        attributes.alignment_scope = wfa::alignment_scope_t_compute_score;
        attributes.linear_penalties.mismatch = sub as i32;
        attributes.linear_penalties.indel = indel as i32;

        if biwfa {
            attributes.memory_mode = wfa::wavefront_memory_t_wavefront_memory_ultralow;
        }

        // Initialize Wavefront Aligner
        let wf_aligner = wfa::wavefront_aligner_new(&mut attributes);
        let a: &[i8] = transmute(a);
        let b: &[i8] = transmute(b);
        let status = wfa::wavefront_align(
            wf_aligner,
            a.as_ptr(),
            a.len() as i32,
            b.as_ptr(),
            b.len() as i32,
        );
        assert_eq!(status, 0);
        let cost = (-(*wf_aligner).cigar.score) as Cost;
        wfa::wavefront_aligner_delete(wf_aligner);
        cost
    }
}

fn affine_cost(
    a: Seq,
    b: Seq,
    mismatch: Cost,
    gap_open: Cost,
    gap_extend: Cost,
    biwfa: bool,
) -> Cost {
    // Configure alignment attributes
    unsafe {
        let mut attributes = wfa::wavefront_aligner_attr_default;
        attributes.heuristic.strategy = wfa::wf_heuristic_strategy_wf_heuristic_none;
        attributes.distance_metric = wfa::distance_metric_t_gap_affine;
        attributes.affine_penalties.mismatch = mismatch as i32;
        attributes.affine_penalties.gap_opening = gap_open as i32;
        attributes.affine_penalties.gap_extension = gap_extend as i32;
        attributes.alignment_scope = wfa::alignment_scope_t_compute_score;
        if biwfa {
            attributes.memory_mode = wfa::wavefront_memory_t_wavefront_memory_ultralow;
        }
        let a: &[i8] = transmute(a);
        let b: &[i8] = transmute(b);
        let wf_aligner = wfa::wavefront_aligner_new(&mut attributes);
        let status = wfa::wavefront_align(
            wf_aligner,
            a.as_ptr(),
            a.len() as i32,
            b.as_ptr(),
            b.len() as i32,
        );
        assert_eq!(status, 0);
        let cost = (-(*wf_aligner).cigar.score) as Cost;
        wfa::wavefront_aligner_delete(wf_aligner);
        cost
    }
}

fn double_affine_cost(
    a: Seq,
    b: Seq,
    mismatch: Cost,
    gap_open1: Cost,
    gap_extend1: Cost,
    gap_open2: Cost,
    gap_extend2: Cost,
    biwfa: bool,
) -> Cost {
    // Configure alignment attributes
    unsafe {
        let mut attributes = wfa::wavefront_aligner_attr_default;
        attributes.heuristic.strategy = wfa::wf_heuristic_strategy_wf_heuristic_none;
        attributes.distance_metric = wfa::distance_metric_t_gap_affine_2p;
        attributes.affine2p_penalties.mismatch = mismatch as i32;
        attributes.affine2p_penalties.gap_opening1 = gap_open1 as i32;
        attributes.affine2p_penalties.gap_extension1 = gap_extend1 as i32;
        attributes.affine2p_penalties.gap_opening2 = gap_open2 as i32;
        attributes.affine2p_penalties.gap_extension2 = gap_extend2 as i32;
        attributes.alignment_scope = wfa::alignment_scope_t_compute_score;
        if biwfa {
            attributes.memory_mode = wfa::wavefront_memory_t_wavefront_memory_ultralow;
        }
        let a: &[i8] = transmute(a);
        let b: &[i8] = transmute(b);
        let wf_aligner = wfa::wavefront_aligner_new(&mut attributes);
        let status = wfa::wavefront_align(
            wf_aligner,
            a.as_ptr(),
            a.len() as i32,
            b.as_ptr(),
            b.len() as i32,
        );
        assert_eq!(status, 0);
        let cost = (-(*wf_aligner).cigar.score) as Cost;
        wfa::wavefront_aligner_delete(wf_aligner);
        cost
    }
}

impl<const N: usize> Aligner for WFA<AffineCost<N>> {
    fn cost(&mut self, a: Seq, b: Seq) -> Cost {
        let mut cost = (|| {
            if N == 0 {
                //lcs cost
                if self.cm.sub == None && self.cm.ins == self.cm.del{
                return lcs_cost(a, b, self.biwfa);
                //unit cost
            } else if self.cm.sub == Some(1) && self.cm.ins == Some(1) && self.cm.del == Some(1){
                return unit_cost(a, b, self.biwfa);
                //linear cost
            } else if let Some(sub) = self.cm.sub
            && let Some(ins) = self.cm.ins
            && let Some(del) = self.cm.del
            && ins == del {
                return linear_cost(a, b, sub, ins, self.biwfa);
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
                        self.cm.affine[0].extend,
                        self.biwfa
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
                        self.cm.affine[2].extend,
                        self.biwfa
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
        let mut biwfa = WFA {
            cm: cm.clone(),
            biwfa: true,
        };
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
