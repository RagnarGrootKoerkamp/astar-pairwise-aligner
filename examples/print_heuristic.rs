use itertools::Itertools;
use pairwise_aligner::prelude::*;

fn main() {
    let pruning = false;
    let n = 11;
    let e: f32 = 0.1;
    let r = 155;
    let k = 4;
    let max_match_cost = 0;

    let h = CSH {
        match_config: MatchConfig {
            length:
            //Fixed(6),
            LengthConfig::Fixed(k),
            max_match_cost,
            ..MatchConfig::default()
        },
        pruning,
        use_gap_cost: false,
        c: PhantomData::<BruteForceContours>::default(),
    };

    let (ref a, ref b, alphabet, _stats) = setup_with_seed(n, e, r);
    let ref a = "CCTCTC".bytes().collect_vec();
    let ref b = "CCCCC".bytes().collect_vec();

    println!("{}\n{}", to_string(a), to_string(b));
    let mut h = h.build(&a, &b, &alphabet);
    h.display(Pos::from_length(a, b), None, None, None, None);
    let graph = AlignmentGraph::new(a, b, true);
    let (distance_and_path, astar) = astar::astar(&graph, Pos(0, 0), &mut h);
    let (distance, path): (u32, Vec<Pos>) = distance_and_path.unwrap_or_default();
    let dist = bio::alignment::distance::simd::levenshtein(&a, &b);
    assert_eq!(distance, dist);
    // h.display(
    //     Pos::from_length(a, b),
    //     Some(distance),
    //     Some(astar.explored_states),
    //     Some(astar.expanded_states),
    //     Some(path),
    // );
}
