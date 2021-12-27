use pairwise_aligner::{prelude::*, *};
use rand::{prelude::Distribution, SeedableRng};

fn main() {
    let pruning = true;
    let make_consistent = false;
    let (l, max_match_cost) = (7, 1);
    for do_transform in [false, true] {
        for build_fast in [false, true] {
            let h_slow = SeedHeuristic {
                l,
                max_match_cost,
                distance_function: GapHeuristic,
                pruning,
                build_fast: false,
                query_fast: false,
                make_consistent,
            };
            let h_fast = SeedHeuristic {
                l,
                max_match_cost,
                distance_function: GapHeuristic,
                pruning,
                build_fast,
                query_fast: false,
                make_consistent,
            };

            let n = 1000;
            let e: f32 = 0.3;
            let (ref a, ref b, alphabet, stats) = setup(n, e);
            let start = 658;
            let end = 756;
            let a = &a[start..end].to_vec();
            let b = &b[start..end].to_vec();
            // let a = &"GAAGGGTAACAGTGCTCG".as_bytes().to_vec();
            // let b = &"AGGGTAACAGTGCTCGTA".as_bytes().to_vec();
            // let (a, b) = (
            //     &"GATCGCAGCAGAACTGTGCCCATTTTGTGCCT".as_bytes().to_vec(),
            //     &"CGGATCGGCGCAGAACATGTGGTCCAATTTTGCTGCC".as_bytes().to_vec(),
            // );
            // let (a, b) = (
            //     &"GCCTAAATGCGAACGTAGATTCGTTGTTCC".as_bytes().to_vec(),
            //     &"GTGCCTCGCCTAAACGGGAACGTAGTTCGTTGTTC".as_bytes().to_vec(),
            // );

            println!("\n\n\nTESTING: {:?}", h_fast);
            println!("{}\n{}", to_string(a), to_string(b));
            let mut h = h_fast.build(&a, &b, &alphabet);
            for p in [
                Pos(21, 8),
                Pos(28, 15),
                Pos(35, 22),
                Pos(42, 29),
                Pos(63, 52),
                Pos(70, 61),
            ] {
                h.prune(p);
            }

            let mut ps = HashMap::default();
            let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(3144);
            let dist = rand::distributions::Uniform::new_inclusive(0u8, 255u8);

            let mut pixels = vec![vec![(None, None, false, false); 20 * b.len()]; 20 * a.len()];

            let start_i = Pos(
                b.len() * (max_match_cost + 1) / (l + max_match_cost + 1) + a.len() - b.len(),
                0,
            );
            let start_j = Pos(0, a.len() * (max_match_cost + 1) / l + b.len() - a.len());
            let start = Pos(h.transform(start_j).0, h.transform(start_i).1);
            let target = Pos(a.len(), b.len());
            print!(
                "target {:?} / {:?}\nstarti {:?}\nstartj {:?}\n",
                target,
                h.transform(target),
                start_i,
                start_j
            );
            // print!(
            //     "target {:?}\nstarti {:?}\nstartj {:?}\nSTART: {:?}\n",
            //     transform(target),
            //     transform(start_i),
            //     transform(start_j),
            //     start
            // );

            let target = h.transform(target);
            //let target = Pos(target.0 + a.len(), target.1 + b.len());

            for i in (0..=a.len()).rev() {
                for j in (0..=b.len()).rev() {
                    let p = Pos(i, j);
                    // Transformation: draw (i,j) at ((l+1)*i + l*(B-j), l*j + (A-i)*(l-1))
                    // scaling: divide draw coordinate by l, using the right offset.
                    let draw_pos = if do_transform { h.transform(p) } else { p };
                    let pixel = &mut pixels[draw_pos.0][draw_pos.1];

                    let (val, parent_pos) = h.base_h_with_parent(p);
                    let l = ps.len();
                    let (_parent_id, color) = ps.entry(parent_pos).or_insert((
                        l,
                        termion::color::Rgb(
                            dist.sample(&mut rng),
                            dist.sample(&mut rng),
                            dist.sample(&mut rng),
                        ),
                    ));
                    let is_start_of_match = h.seed_matches.iter().find(|m| m.start == p).is_some();
                    let is_end_of_match = h.seed_matches.iter().find(|m| m.end == p).is_some();
                    if is_start_of_match {
                        pixel.2 = true;
                    } else if is_end_of_match {
                        pixel.3 = true;
                    }
                    pixel.0 = Some(*color);
                    pixel.1 = Some(val);
                }
            }

            let print = |i: usize, j: usize| {
                let pixel = pixels[i][j];
                if pixel.2 {
                    print!(
                        "{}{}",
                        termion::color::Fg(termion::color::Black),
                        termion::style::Bold
                    );
                } else if pixel.3 {
                    print!(
                        "{}{}",
                        termion::color::Fg(termion::color::Rgb(100, 100, 100)),
                        termion::style::Bold
                    );
                }
                print!(
                    "{}{:3} ",
                    termion::color::Bg(pixel.0.unwrap_or(termion::color::Rgb(0, 0, 0))),
                    pixel.1.map(|x| format!("{:3}", x)).unwrap_or_default()
                );
                print!("{}", termion::color::Fg(termion::color::Reset));
                print!("{}", termion::color::Bg(termion::color::Reset));
            };

            if do_transform {
                for j in start.1..=target.1 {
                    for i in start.0..=target.0 {
                        print(i, j);
                    }
                    print!("{}", termion::color::Fg(termion::color::Reset));
                    print!("{}", termion::color::Bg(termion::color::Reset));
                    print!("\n");
                }
            } else {
                for j in 0 * b.len()..=1 * b.len() {
                    for i in 0 * a.len()..=1 * a.len() {
                        print(i, j);
                    }
                    print!("{}", termion::color::Fg(termion::color::Reset));
                    print!("{}", termion::color::Bg(termion::color::Reset));
                    print!("\n");
                }
            }

            if do_transform {
                println!("\n\n\nALIGN");
                align(
                    &a,
                    &b,
                    &alphabet,
                    stats,
                    EqualHeuristic {
                        h1: h_slow,
                        h2: h_fast,
                    },
                );
            }
        }
    }
}
