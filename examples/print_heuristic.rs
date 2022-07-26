use astar_pairwise_aligner::{prelude::*, visualizer::NoVisualizer};

fn main() {
    let pruning = true;
    let n = 500;
    let e: f32 = 0.3;
    let r = 31415;
    let k = 6;
    let max_match_cost = 1;

    let h = SH {
        match_config: MatchConfig::new(k, max_match_cost),
        pruning,
    };

    let (ref a, ref b, alphabet, _stats) = setup_with_seed(n, e, r);
    let a = &a[186..219];
    let b = &b[186..219];

    println!("{}\n{}", to_string(a), to_string(b));
    let mut h = h.build(&a, &b, &alphabet);
    h.display(Pos::from_lengths(a, b), None, None, None, None, None);
    let graph = EditGraph::new(a, b, true);
    let (distance_and_path, _astar) = astar::astar(&graph, &mut h, &mut NoVisualizer);
    let (distance, _path): (u32, Vec<Pos>) = distance_and_path.unwrap_or_default();
    let dist = bio::alignment::distance::simd::levenshtein(&a, &b);
    assert_eq!(distance, dist);
    h.display(
        Pos::from_lengths(a, b),
        Some(distance),
        Some(_astar.explored_states),
        Some(_astar.expanded_states),
        Some(_path),
        None,
    );
}
