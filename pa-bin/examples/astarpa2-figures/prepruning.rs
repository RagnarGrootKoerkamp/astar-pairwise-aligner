use astarpa::astar_with_vis;
use pa_generate::uniform_seeded;
use pa_heuristic::matches::{prepruning, CenteredVec};
use pa_heuristic::{MatchConfig, Prune, Pruning, CSH, GCSH, SH};
use pa_types::I;
use pa_vis::visualizer::{self, Gradient, When};
use pa_vis::{canvas::*, VisualizerT};
use std::path::PathBuf;
use std::time::Duration;

fn main() {
    let (ref a, ref b) = uniform_seeded(60, 0.30, 31420);
    let mut config = visualizer::Config::default();
    config.draw = When::None;
    config.save = When::None;
    config.save_last = true;
    config.paused = false;
    config.delay = Duration::from_secs_f32(0.0001);
    config.cell_size = 12;
    config.draw_old_on_top = false;
    config.style.bg_color = WHITE;
    config.style.expanded = Gradient::Fixed((130, 179, 102, 0));
    config.style.preprune = Some((130, 179, 102, 0));
    config.style.explored = Some((0, 102, 204, 0));
    config.style.heuristic = Gradient::Gradient((250, 250, 250, 0)..(180, 180, 180, 0));
    config.style.max_heuristic = Some(10);
    config.style.max_layer = Some(16);
    config.style.layer = Gradient::Gradient((250, 250, 250, 0)..(48, 48, 48, 0));
    config.style.pruned_match = RED;
    config.style.path = None;
    config.style.match_width = 3;
    config.style.draw_heuristic = false;
    config.style.draw_layers = true;
    config.style.draw_contours = true;
    config.style.draw_matches = true;
    config.style.contour = BLACK;
    config.layer_drawing = false;
    config.style.draw_dt = false;
    config.style.draw_f = false;
    config.style.draw_labels = false;

    config.filepath = PathBuf::from("imgs/astarpa2-paper/prepruning/");

    if !cfg!(feature = "example") {
        panic!("WARNING: Without the example feature, pruned matches aren't shown red for SH");
    }

    let k = 3;
    let p = 3;
    let mut match_config = MatchConfig {
        length: pa_heuristic::LengthConfig::Fixed(k),
        r: 1,
        local_pruning: 3,
    };

    let prepruned_states = |transform| {
        let mut prepruned = vec![];
        // Some hacking here: we build the matches ourselves up-front to get the
        // positions expanded during prepruning, and then forward those to the actual visualizer.
        let matches = pa_heuristic::matches::find_matches(a, b, match_config, transform);
        let mut next_match_per_diag = CenteredVec::new(a.len() as I - b.len() as I, I::MAX);
        for m in matches.matches.iter().rev() {
            prepruning::preserve_for_local_pruning(
                a,
                b,
                &matches.seeds,
                &m,
                p,
                &mut [vec![], vec![], vec![]],
                &mut next_match_per_diag,
                &mut |p| {
                    prepruned.push(p);
                },
            );
        }
        prepruned
    };
    let prepruned = prepruned_states(false);
    let prepruned_transform = prepruned_states(true);

    for preprune in [false, true] {
        for pruning in [Prune::None, Prune::Both] {
            match_config.local_pruning = if preprune { p } else { 0 };
            let mut suf = (if preprune { "-lp" } else { "" }).to_string();
            suf += if pruning != Prune::None { "-p" } else { "" };
            {
                let mut v = config.with_filename(&("sh".to_string() + &suf)).build(a, b);
                if preprune {
                    v.preprune = prepruned.clone();
                }
                astar_with_vis(a, b, &SH::new(match_config, Pruning::new(pruning)), &mut v);
            }
            {
                let mut v = config
                    .with_filename(&("csh".to_string() + &suf))
                    .build(a, b);
                if preprune {
                    v.preprune = prepruned.clone();
                }
                astar_with_vis(a, b, &CSH::new(match_config, Pruning::new(pruning)), &mut v);
            }
            {
                let mut v = config
                    .with_filename(&("gcsh".to_string() + &suf))
                    .build(a, b);
                if preprune {
                    v.preprune = prepruned_transform.clone();
                }
                astar_with_vis(
                    a,
                    b,
                    &GCSH::new(match_config, Pruning::new(pruning)),
                    &mut v,
                );
            }
        }
    }
}
