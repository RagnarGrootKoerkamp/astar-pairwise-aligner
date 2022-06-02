// To create a video from images use this command:

// ffmpeg -framerate 4 -i %d.bmp -vf fps=4 -pix_fmt yuv420p output1.mp4

// (You need to have ffmpeg installed. And make sure that binary is in the folder that is included in PATH (I have no idea tbh does Mac have PATH or not. Maybe this thing with PATH is only for windows))

// Sometimes there can be an error like this: height(or width) not divisible by 2. Use this command in this case:

// ffmpeg -framerate 4 -i %d.bmp -vf "pad=ceil(iw/2)*2:ceil(ih/2)*2" -pix_fmt yuv420p output1.mp4

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
        let graph = EditGraph::new(a, b, true);
        let tmp = config.filepath.clone();
        config.filepath = format!("{}{}", &config.filepath, "/SH/");
        let (distance_and_path, _astar) = astar::astar(&graph, Pos(0, 0), &mut h, &config);
        config.filepath = tmp;
        let (_distance, _path) = distance_and_path.unwrap_or_default();
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
        let graph = EditGraph::new(a, b, true);
        let tmp = config.filepath.clone();
        config.filepath = format!("{}{}", config.filepath, "/CSH/");
        let (distance_and_path, _astar) = astar::astar(&graph, Pos(0, 0), &mut h, &config);
        config.filepath = tmp;
        let (_distance, _path) = distance_and_path.unwrap_or_default();
    }
}
