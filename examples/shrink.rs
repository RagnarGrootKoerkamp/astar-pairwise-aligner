use pairwise_aligner::prelude::*;

fn main() {
    let n = 1000;
    let e = 0.20;
    let l = 7;
    let max_match_cost = 1;
    let pruning = true;

    let h = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(l),
            max_match_cost,
            ..MatchConfig::default()
        },
        pruning,
        prune_fraction: 0.5,
        c: PhantomData::<NaiveContours<LogQueryContour>>,
        ..GapSeedHeuristic::default()
    };

    let (_a, _b, alphabet, stats) = setup(n, e);
    let a = "TTGCATGGAAAGCTTGCTTATCTGTTTTCTCCTTGTTCGTCGGGCGAGGATCCGAAGTAAGCCGCTGCAAAGTTACCCATCAATGATCATCTCGCTGTAGACCAACCGGCCAGGTATTCTTTGCCTATTCGTTTATCCGATAACGAGGTACGTAAAAAATGGTCCTCTCTTATTGGAAATCGGAGGCGTCGGACTGCCCGAAGAACGACTTTCGAACCTATCTTGGCTAGGATACTCATTTTGCCGGGACTTGGTCCCCCATGTAAGGAGGTCCCTATGACCAAATCGCTACTGAAGTGTCATTGAGGAAGCGTCGTTCATCCTAGTTCGGGTCGCGATGAGATGGCCGTGGATCACCTTTGGGTAACATACGCAATGCGCTACTGGCTGATAGCAGCGTCTATACTATCGCCTTACCGAAGTCCCCATATCTTAACAAACTCTGCAAGGCTACAGGGGGTCTTCATGAAGGAAGTCGTAAAACGATATCTGTGCATCTTGGGGGACAGCCTGCAAGACCGAACTGGTACTGTCCCGTGCAAATCCTTCCCTGCCGATCAAATTGACATAGAATTCATCGAGCGATACGGCGGAGAATAATTAAGACGATCAGTTTGAAAGCTCGCCTAGTACAATCGGCCTGAGCTAGTCCAAGTCTGGCGTTTAGAGCCAATATCTAAGCCTCTTTTCCAACTTCGTATCAACCTGAATTCATTTGGGAATACTTTCTTCAAGGCCCGCAGTACGAAATACGAGCAGTACCACTTTCACTATGACAACCTCGGGCATTTTCCATATGCTTATTTAATCCCCTATGGTCGTCTCCGCCCTCATCTTCTGTCAAAATGTTACTCCCTAGACCCGCGCAACTTGATTCTCATCGTTTAGTTTCCCTTAAACGAAAAATGACGATAAGACCACCCGTAGACGCTGGATTACACACCCCAGTGGATTGTAATCTACGTCGGGAGAATCGCCAAATAAGTTTAGATTCCGTACC".as_bytes().to_vec();
    let b = "TTACTGTATGAAAGGCTTGGTTATCTGCGGCCTTGTTCGTCGGGCCGGATCCGAAGTTAAGCCGCTGCCAAAGAACACCCTTCAATGATCGCTCGCGTGACCCACCGCCGGGTTTTCTTTCCTATTCGTTTTATACCCATAATAGTACGTAAAAAATTGGTCATCTTCTATTTGGACAATCGGAGTCGTCGGACTCCGGAAAACGACTTTCGAACCTATCTTGGCTAGGGATACTTCAGTTTTGCCGGGGACTCTGCCCCCCCATATAAGGAGTCCCTACGACCAAATCGCTACTTAAGTGTCATTGAGTAAGCGGTCTTCACCTAGTTCGGGTCGCGATGCGGATGGCCGATGGATCACACTTTCGCGTAAATACGGAATAGTCGGCTACTGGGCCTCATAGACGTTCGATCTATCGCTTACCGAAGTTCACCATATTTAACAAACTCTGACAAGGCTACAGCGGGGTCTCATGAAGAGAAGTCGTAAAACGGGATCGTGCATCTTGGGGGACAGCCTGCTAGACCAGAACTGGTACTGTCCCGTGAAATCCCTCCCTGCACGATCAAATTGACATAGAGTCACGAGCGATACGGCGGACGTATATGGAATAAGAGGATCAGTTTGAAAGCGTCGCCTAGTACAATCGGCCTGAGCTAGTCCAATCTGGCGTTTAAGACCAACATCTAGCCTCACTTCCAACTTCGTGTTCAACCTGATACTCCAGTTTGGGAATGACTTTCTTCAAGGCCCTTGCAGTACGAAATACGAGCAGTCCGCATTTCAACTATGGCACCTCCGGGCAATTAACATATGATTATTTAATCCCCTATGGTTCAGTCTCCGCCCTCACTCTCCTGTCAACAATGTTACTCCCTAGACCCGCGCAACTTAATTCTCATCGTTTAGTTTCCCGTTAAACGAAAAAGGACGATAAGAACCCACCCGTAGACGCTGGATTATACACCACTAGGTGGATTGTATCTACGTCGGAGAATCGCCAAATAGGTTAGATTCCGGTAGA".as_bytes().to_vec();
    println!("Heuristic:\n{:?}", h);
    println!("{}\n{}", to_string(&a), to_string(&b));
    align(&a, &b, &alphabet, stats, h).print();
    return;

    {
        // True on success.
        let test = |start, end| {
            std::panic::catch_unwind(|| {
                align(
                    &a[start..min(a.len(), end)].to_vec(),
                    &b[start..min(b.len(), end)].to_vec(),
                    &alphabet,
                    stats,
                    h,
                )
                .print()
            })
            .is_ok()
        };
        let start;
        let mut end = max(a.len(), b.len());

        // Binary search the start of the sequence in steps of l.
        {
            let mut left = 0;
            let mut right = end;
            while left / l < right / l {
                let mid = (left + right) / 2 / l * l;
                if test(mid, end) {
                    right = mid;
                } else {
                    left = mid;
                }
            }
            start = left;
        }
        // Binary search the end of the sequence.
        {
            let mut left = start;
            let mut right = end;
            while left < right {
                let mid = (left + right) / 2;
                if test(start, mid) {
                    left = mid;
                } else {
                    right = mid;
                }
            }
            end = left;
        }
        assert!(!test(start, end));
    }
}
