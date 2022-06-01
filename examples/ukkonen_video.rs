use std::cell::Cell;

use num_traits::abs;
use pairwise_aligner::{astar::Config, prelude::*, ukkonen::ukkonen, ukkonen::ukkonen_vis};

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

    let n = 40;
    let e = 0.25;

    let m = 0;
    let k = 3;

    let (ref a, ref b, ref alphabet, stats) = setup_with_seed(n, e, 1528);

    let _target = Pos::from_length(&a, &b);
    print!("{}\n{}\n", to_string(a), to_string(b));
    /*let hmax = Some(align(a, b, alphabet, stats, ZeroCost).edit_distance);
    config.hmax = hmax;*/

    config.hmax = None;

    /*{
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
    }*/

    //Ukkonen

    let tmp = config.filepath.clone();
    config.filepath = format!("{}{}", config.filepath, "/Ukkonen/");
    let mut d = max(2, abs(a.len() as i32 - b.len() as i32) as usize);
    let mut r = d + 1;
    let start = std::time::Instant::now();
    let mut file_number = 0;
    let mut is_playing: bool = false;
    let mut skip = 0;
    let mut prev = vec![vec![]];
    while r > d {
        (r, file_number, is_playing, skip) =
            ukkonen_vis(a, b, d, &config, file_number, is_playing, skip, &mut prev);

        println!("d = {} r = {}", d, r);
        d *= 2;
        r *= 2;
    }
    let duration = start.elapsed().as_secs_f32();

    println!("Ukkonen says that edit distance is {}", r / 2);

    println!("Ukkonen has needed for this {duration} seconds");

    config.filepath = tmp;
}
