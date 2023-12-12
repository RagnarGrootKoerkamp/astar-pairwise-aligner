//! Tests that test A*PA with various configurations.
use crate::AstarPa;
use itertools::Itertools;
use pa_affine_constants::{INDEL_COST, SUB_COST};
use pa_heuristic::*;
use pa_types::*;
use pa_types::{Cost, Seq};
use pa_vis_types::NoVis;

fn scaled_levenshtein(a: &[u8], b: &[u8], indel_cost: Cost, sub_cost: Cost) -> Cost {
    let mut prev = (0..a.len() as i32 + 1)
        .map(|i| i * indel_cost as Cost)
        .collect_vec();
    let mut next = vec![0; a.len() + 1];

    for i in 1..b.len() + 1 {
        next[0] = (i + 1) as Cost * indel_cost;
        for j in 1..a.len() + 1 {
            next[j] = (prev[j - 1] + (a[j - 1] != b[i - 1]) as Cost * sub_cost)
                .min(prev[j] + indel_cost)
                .min(next[j - 1] + indel_cost);
        }
        std::mem::swap(&mut prev, &mut next);
    }

    prev[a.len()]
}

fn affine_test_aligner_on_input(a: Seq, b: Seq, aligner: &mut impl Aligner, params: &str) {
    // Set to true for local debugging.
    const D: bool = false;

    // useful in case of panics inside the alignment code.
    eprintln!("{params}");
    if D {
        eprintln!("a {}\nb {}", seq_to_string(a), seq_to_string(b));
    }

    let cost = INDEL_COST.with(|indel_cost| {
        SUB_COST.with(|sub_cost| {
            scaled_levenshtein(&a, &b, *indel_cost.borrow(), *sub_cost.borrow()) as Cost
        })
    });
    let aligner_cost = aligner.align(a, b).0;
    // Test the cost reported by all aligners.
    assert_eq!(
        cost,
        aligner_cost,
        "\n{params}\nlet a = \"{}\".as_bytes();\nlet b = \"{}\".as_bytes();\nAligner\n{aligner:?}",
        seq_to_string(&a),
        seq_to_string(&b),
    );
    let (cost, Some(cigar)) = aligner.align(a, b) else {
        // Cigar not returned so not cheked.
        return;
    };
    if cost != cost {
        eprintln!("\n================= TEST CIGAR ======================\n");
        eprintln!(
            "{params}\nlet a = \"{}\".as_bytes();\nlet b = \"{}\".as_bytes();\ncigar: {}",
            seq_to_string(a),
            seq_to_string(b),
            cigar.to_string(),
        );
    }
    assert_eq!(cost, cost);
    cigar.verify(&CostModel::unit(), a, b);
}

fn set_indel_cost(indel_cost: Cost) {
    INDEL_COST.with(|indel_cost_cell| {
        *indel_cost_cell.borrow_mut() = indel_cost;
    });
}

fn set_sub_cost(sub_cost: Cost) {
    SUB_COST.with(|sub_cost_cell| {
        *sub_cost_cell.borrow_mut() = sub_cost;
    });
}

mod affine_edge_cases {
    use super::*;

    #[test]
    fn indel_cost_in_transform_small() {
        set_indel_cost(3);
        set_sub_cost(2);

        let aligner = &mut AstarPa {
            dt: true,
            h: AffineBruteGCSH::new(MatchConfig::new(2, 1), Pruning::start()),
            v: NoVis,
        };

        let a = "AGTT".as_bytes();
        let b = "AGT".as_bytes();
        affine_test_aligner_on_input(a, b, aligner, "");
    }

    #[test]
    fn indel_cost_in_transform_no_filtered_matches() {
        set_indel_cost(3);
        set_sub_cost(2);

        let aligner = &mut AstarPa {
            dt: true,
            h: AffineBruteGCSH::new(MatchConfig::new(2, 1), Pruning::start()),
            v: NoVis,
        };

        let a = "GTTA".as_bytes();
        let b = "GGTTA".as_bytes();
        affine_test_aligner_on_input(a, b, aligner, "");
    }

    #[test]
    fn indel_cost_in_transform_big() {
        set_indel_cost(3);
        set_sub_cost(2);

        let aligner = &mut AstarPa {
            dt: true,
            h: AffineBruteGCSH::new(MatchConfig::new(2, 1), Pruning::start()),
            v: NoVis,
        };

        let a = "TGGAACCCCATCGCAAATCACCCCACAGGTAACGTCATAACTACCGCATGGTACGGTACCCCTTCTGCGATAGAGATGGT"
            .as_bytes();
        let b = "TTGGAACCCACATCGCAAATCACCCCACAGGTAACGTCATAACTACCGCATGGTTCGGGTACCCCTTCGTGCGATAGAGA"
            .as_bytes();
        affine_test_aligner_on_input(a, b, aligner, "");
    }

    #[test]
    fn sub_cost_in_transform_small() {
        set_indel_cost(2);
        set_sub_cost(3);

        let aligner = &mut AstarPa {
            dt: true,
            h: AffineBruteGCSH::new(MatchConfig::new(2, 1), Pruning::start()),
            v: NoVis,
        };

        let a = "AGTT".as_bytes();
        let b = "AGT".as_bytes();
        affine_test_aligner_on_input(a, b, aligner, "");
    }

    #[test]
    fn sub_cost_in_transform_no_filtered_matches() {
        set_indel_cost(2);
        set_sub_cost(3);

        let aligner = &mut AstarPa {
            dt: true,
            h: AffineBruteGCSH::new(MatchConfig::new(2, 1), Pruning::start()),
            v: NoVis,
        };

        let a = "GTTA".as_bytes();
        let b = "GGTTA".as_bytes();
        affine_test_aligner_on_input(a, b, aligner, "");
    }

    #[test]
    fn sub_cost_in_transform_big() {
        set_indel_cost(2);
        set_sub_cost(3);

        let aligner = &mut AstarPa {
            dt: true,
            h: AffineBruteGCSH::new(MatchConfig::new(2, 1), Pruning::start()),
            v: NoVis,
        };

        let a = "TGGAACCCCATCGCAAATCACCCCACAGGTAACGTCATAACTACCGCATGGTACGGTACCCCTTCTGCGATAGAGATGGT"
            .as_bytes();
        let b = "TTGGAACCCACATCGCAAATCACCCCACAGGTAACGTCATAACTACCGCATGGTTCGGGTACCCCTTCGTGCGATAGAGA"
            .as_bytes();
        affine_test_aligner_on_input(a, b, aligner, "");
    }
}
