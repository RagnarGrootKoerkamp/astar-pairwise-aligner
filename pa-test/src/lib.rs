use itertools::Itertools;
use rand::{seq::IteratorRandom, thread_rng, Rng};

use pa_generate::ErrorModel;
use pa_types::*;

fn test_sequences() -> Vec<(Seq<'static>, Seq<'static>)> {
    vec![
        (b"TTGGGTCAATCAGCCAGTTTTTA", b"TTTGAGTGGGTCATCACCGATTTTAT"),
        (b"ACTGACCAGT", b"CCGACAGGA"),
        (b"AGTTTTAT", b"ACCGATTTTTA"),
        (b"CTCTCTTCTCTCTCTA", b"CCTCTCTCTCTCCTCTC"),
        (b"AGTGGGTTGCCTTCATTCCG", b"AGTGGTGTCTTCAGGCCTTCATTCCG"),
        (b"GCACGTCGCCCCCCGCCCGCG", b"GCCCGCCCGCCCGCCCCCGCCCCC"),
        (b"CGCGTGTATCCGTCCACATCGAGCCGCCCTTGTTGCTTTTCGAGCGCTCATTTCCCGCAAGAGTGGCGTGCGGTCACTTTCGCGCAGCAATTAGAGTACTAACGGGTAGACGTGGCTTTCCTCCTCGTCCTGTCAACGCGCATAGGATGTCCTGCAGCAGGCCGCCGCGATTGCCTAAATCAAGGGGTTCCAATGGAGTTTCCATCTGATATCCGCGCTCCGGTTCTGAGTCTAAAGTGGAAATACTCCGAATGGGCCGGTATGAGGTTGGGTCAATCAGCCAGTTTTTA",
         b"CGCTGGGGATGCCTCCACCTTTCGAGTGCCTGTTGGTTCCGACGCTATCATAGTCCCCATGCAAGGAGATGGCTGCGCGTCCTATCGCGCGGCAAATAGAGTCTACGGGGGCGGCTGTCCTCCTCGTCCTGGTCAACGGCCATAGGATTTCCGCGATGGTCGCCCGGATGTGCCTAAACCAAGGCTCCGATGGAGCTGCCTCTGATATCCGCGCTGCCGGTTTCCTGACGTCTGAAAACGTTGGAAAATACCTCCGAATGGGCCCCGTTTGAGTGGGTCATCACCGATTTTAT"),
    ]
}

const FIXED: bool = false;

pub fn gen_seqs() -> impl Iterator<Item = ((Sequence, Sequence), (usize, f32, ErrorModel, u64))> {
    let rng = &mut thread_rng();
    let mut ns = vec![
        0usize, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 30, 40, 50,
        60, 70, 80, 90, 100, 110, 120, 130, 140, 150, 160, 170, 180, 190, 200, 210, 220, 230, 240,
        250, 254, 255, 256, 257, 258, 260, 270, 280, 290, 300, 500, 511, 512, 513, 515,
    ];
    let mut es = vec![
        0.0f32, 0.01, 0.02, 0.03, 0.05, 0.10, 0.20, 0.30, 0.40, 0.50, 0.60, 0.70, 1.0,
    ];

    // Pick a random subset of the above for 4x speedup. CI runs often enough to get good coverage.
    if !FIXED {
        let nl = ns.len();
        ns = ns.into_iter().choose_multiple(rng, nl / 4);
        let el = es.len();
        es = es.into_iter().choose_multiple(rng, el / 4);
    }

    let models = [
        ErrorModel::Uniform,
        ErrorModel::NoisyInsert,
        ErrorModel::NoisyDelete,
        ErrorModel::SymmetricRepeat,
    ];
    // Run each test on a new random seed for increased coverage over time.
    let seeds = if FIXED {
        [31415]
    } else {
        [rng.gen_range(0..u64::MAX)]
    };
    ns.into_iter()
        .cartesian_product(es)
        .cartesian_product(models)
        .cartesian_product(seeds)
        .map(|(((n, e), error_model), seed)| {
            let (a, b) = pa_generate::generate_model(n, e, error_model, seed);
            ((a, b), (n, e, error_model, seed))
        })
}

pub fn test_aligner_on_input(a: Seq, b: Seq, aligner: &mut impl Aligner, params: &str) {
    // Set to true for local debugging.
    const D: bool = false;

    // useful in case of panics inside the alignment code.
    eprintln!("{params}");
    if D {
        eprintln!("a {}\nb {}", seq_to_string(a), seq_to_string(b));
    }
    let cost = triple_accel::levenshtein_exp(&a, &b) as Cost;
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
        // Cigar not returned so not checked.
        return;
    };
    if cost != aligner_cost {
        eprintln!("\n================= TEST CIGAR ======================\n");
        eprintln!(
            "{params}\nlet a = \"{}\".as_bytes();\nlet b = \"{}\".as_bytes();\ncigar: {}",
            seq_to_string(a),
            seq_to_string(b),
            cigar.to_string(),
        );
    }
    assert_eq!(cost, aligner_cost);
    cigar.verify(&CostModel::unit(), a, b);
}

/// Test the given aligner on a large set of random sequences:
/// - length 1 to 1000
/// - error rate 0.01 to 1.0
/// - error models: uniform, noisy insert, noisy delete, symmetric repeat (using `pa_generate`)
///
/// - The cost reported by the aligner must match `triple_accel::levenshtein_exp`.
/// - The returned cigar must have the right cost and be valid.
pub fn test_aligner(aligner: impl Aligner) {
    test_aligner_up_to(aligner, usize::MAX);
}

/// As test_aligner, but only test sequences with n <= max_n.
pub fn test_aligner_up_to(mut aligner: impl Aligner, max_n: usize) {
    for (a, b) in test_sequences() {
        test_aligner_on_input(
            &a,
            &b,
            &mut aligner,
            &format!(
                "hardcoded test_sequences: a {:?} b {:?}",
                seq_to_string(&a),
                seq_to_string(&b)
            ),
        );
    }
    for ((a, b), (n, e, error_model, seed)) in gen_seqs() {
        if n > max_n {
            continue;
        }
        test_aligner_on_input(
            &a,
            &b,
            &mut aligner,
            &format!("seed {seed:>10} n {n:>5} e {e:>.2} error_model {error_model:?}"),
        );
    }
}
