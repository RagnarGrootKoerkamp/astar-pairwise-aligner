use pairwise_aligner::prelude::*;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(short, default_value = "10")]
    k: I,
    #[structopt(short, default_value = "10000")]
    n: usize,
    #[structopt(short, default_value = "0.2")]
    e: f32,
    #[structopt(short = "x", default_value = "50")]
    samples: usize,
}

fn main() {
    let args = Cli::from_args();
    let (a, b, alph, _) = setup_with_seed(args.n, args.e, 31415);
    let k = args.k;
    let max_match_cost = 0;
    fixed_seeds(&RankTransform::new(&alph), max_match_cost, &a, k);

    let trie = Trie::new(
        b.windows(k as usize)
            .enumerate()
            .map(|(i, w)| (w, i as crate::datastructures::trie::Data)),
        &alph,
    );

    // Find the closest match for each kmer in a.
    let mut pot = 0 as usize;
    let mut capped = [0 as usize; 9];
    for slice in a.chunks(k as usize).take(args.samples) {
        let mut first = 0;
        let mut i = 0;
        let mut cnt = 0;
        for cost in 0.. {
            trie.matches(slice, cost, |start, _, _| {
                if cnt == 0 {
                    first = cost;
                    i = start;
                } else {
                    if abs_diff(start, i) <= 2 * k {
                        return;
                    }
                }
                cnt += 1;
            });
            if cnt > 1 {
                pot += cost as usize;
                for (i, c) in capped.iter_mut().enumerate() {
                    *c += min(i, cost as usize);
                }
                println!("First at {first} \t then {cost}+");
                break;
            }
        }
    }
    println!("Edit distance         : {:6.3}", args.e);

    let covered = args.samples * args.k as usize;
    for (i, c) in capped.iter().enumerate() {
        println!(
            "Capped potential      : {:6.3}  at {i:2}",
            *c as f32 / covered as f32
        );
    }
    println!(
        "Max potential per char: {:6.3}",
        pot as f32 / covered as f32
    );
}
