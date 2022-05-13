use std::cell::Cell;

use pairwise_aligner::{astar::Config, prelude::*};

fn parse_config(args: &[String]) -> (bool, String) {
    if args.len() < 2 {
        print!("Video will not be saved for the filepath was not provided");
        return (false, String::default());
    }

    (true, args[1].clone())
}

fn main() {
    let mut config = Config {
        saving: true,
        filepath: String::from(""),
        drawing: true,
        delay: Cell::new(0.2),
        hmax: Some(0),
    };
    let args: Vec<String> = std::env::args().collect();

    (config.saving, config.filepath) = parse_config(&args);

    let n = 60;
    let e = 0.25;

    let m = 0;
    let k = 3;

    let (ref a, ref b, ref alphabet, stats) = setup_with_seed(n, e, 1524);

    let _target = Pos::from_length(&a, &b);
    let hmax = Some(align(a, b, alphabet, stats, ZeroCost).edit_distance);
    config.hmax = hmax;

    // {
    //     let h = ZeroCost;
    //     let h = Heuristic::build(&h, &a, &b, &alphabet);
    //     h.display(target, DisplayType::Heuristic, hmax);
    // }
    // {
    //     let h = PerfectHeuristic;
    //     let h = Heuristic::build(&h, &a, &b, &alphabet);
    //     h.display(target, DisplayType::Heuristic, hmax);
    // }
    // {
    //     let h = GapCost;
    //     let h = Heuristic::build(&h, &a, &b, &alphabet);
    //     h.display(target, DisplayType::Heuristic, hmax);
    // }
    // {
    //     let h = CountCost;
    //     let h = Heuristic::build(&h, &a, &b, &alphabet);
    //     h.display(target, DisplayType::Heuristic, hmax);
    // }
    {
        let h = SH {
            match_config: MatchConfig {
                length: Fixed(k),
                max_match_cost: m,
                ..MatchConfig::default()
            },
            pruning: true,
        };
        let mut h = Heuristic::build(&h, &a, &b, &alphabet);
        //h.display(target, hmax, None, None, None);
        let graph = EditGraph::new(a, b, true);
        let tmp = config.filepath.clone();
        config.filepath = format!("{}{}", &config.filepath, "/SH/");
        let (distance_and_path, _astar) = astar::astar(&graph, Pos(0, 0), &mut h, &config);
        config.filepath = tmp;
        let (_distance, _path) = distance_and_path.unwrap_or_default();

        // h.display(
        //     target,
        //     hmax,
        //     Some(astar.explored_states),
        //     Some(astar.expanded_states),
        //     Some(path),
        //     Some(astar.tree),
        // );
    }
    {
        let h = CSH {
            match_config: MatchConfig {
                length: Fixed(k),
                max_match_cost: m,
                ..MatchConfig::default()
            },
            pruning: true,
            use_gap_cost: false,
            c: PhantomData::<HintContours<BruteForceContour>>::default(),
        };
        let mut h = Heuristic::build(&h, &a, &b, &alphabet);
        //h.display(target, hmax, None, None, None);
        let graph = EditGraph::new(a, b, true);
        let tmp = config.filepath.clone();
        config.filepath = format!("{}{}", config.filepath, "/CSH/");
        let (distance_and_path, _astar) = astar::astar(&graph, Pos(0, 0), &mut h, &config);
        config.filepath = tmp;
        let (_distance, _path) = distance_and_path.unwrap_or_default();

        /*h.display(
            target,
            hmax,
            Some(&astar.explored_states),
            Some(&astar.expanded_states),
            Some(&path),

                Some(astar.tree),
        );*/
    }
}
