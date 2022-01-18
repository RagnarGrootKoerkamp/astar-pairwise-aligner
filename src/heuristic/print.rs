use rand::{prelude::Distribution, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::io;

use crate::prelude::*;

pub fn print<'a, 'b, H>(
    h: &H,
    matches: impl Iterator<Item = &'b Match>,
    target: Pos,
    wait_for_user: bool,
) where
    H: HeuristicInstance<'a, Pos = Pos>,
{
    if !crate::config::print() {
        return;
    }
    let mut matches_by_start = HashSet::default();
    let mut matches_by_end = HashSet::default();
    for m in matches {
        matches_by_start.insert(m.start);
        matches_by_end.insert(m.end);
    }

    let mut ps = HashMap::default();
    let mut rng = ChaCha8Rng::seed_from_u64(3144);
    let dist = rand::distributions::Uniform::new_inclusive(0u8, 255u8);
    let Pos(a, b) = target;
    let mut pixels = vec![vec![(None, None, false, false); 20 * b as usize]; 20 * a as usize];
    for i in 0..=a {
        for j in 0..=b {
            let p = Pos(i, j);
            let pixel = &mut pixels[p.0 as usize][p.1 as usize];

            let (val, parent_pos) = h.h_with_parent(p);
            let l = ps.len();
            let (_parent_id, color) = ps.entry(parent_pos).or_insert((
                l,
                termion::color::Rgb(
                    dist.sample(&mut rng),
                    dist.sample(&mut rng),
                    dist.sample(&mut rng),
                ),
            ));
            let is_start_of_match = matches_by_start.contains(&p);
            let is_end_of_match = matches_by_end.contains(&p);
            if is_start_of_match {
                pixel.2 = true;
            } else if is_end_of_match {
                pixel.3 = true;
            }
            pixel.0 = Some(*color);
            pixel.1 = Some(val);
        }
    }
    let print = |i: I, j: I| {
        let pixel = pixels[i as usize][j as usize];
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
        print!(
            "{}{}",
            termion::color::Fg(termion::color::Reset),
            termion::color::Bg(termion::color::Reset)
        );
    };
    for j in 0..=b {
        for i in 0..=a {
            print(i, j);
        }
        println!(
            "{}{}",
            termion::color::Fg(termion::color::Reset),
            termion::color::Bg(termion::color::Reset)
        );
    }
    if wait_for_user {
        let mut ret = String::new();
        io::stdin().read_line(&mut ret).unwrap();
    }
}
