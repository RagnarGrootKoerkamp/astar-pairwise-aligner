//! This generates the visualizations used in figure 1 in the paper and in the slides.
#[cfg(not(feature = "sdl2"))]
fn main() {}

#[cfg(feature = "sdl2")]
fn main() {
    use std::path::PathBuf;

    use astar_pairwise_aligner::{
        aligners::{
            astar::Astar,
            diagonal_transition::{DiagonalTransition, GapCostHeuristic},
            nw::NW,
            Aligner,
        },
        prelude::*,
        visualizer::{Gradient, Visualizer, When},
    };
    use instant::Duration;

    let n = 500;
    let e = 0.20;
    let (ref a, ref b) = setup_sequences(n, e);

    let cm = LinearCost::new_unit();
    let mut config = visualizer::Config::default();
    config.draw = When::None;
    config.save = When::Last;
    config.delay = Duration::from_secs_f32(0.0001);
    config.cell_size = 4;
    config.style.bg_color = (255, 255, 255, 128);
    config.style.expanded = Gradient::TurboGradient(0.25..0.90);
    config.style.path_width = None;
    config.draw_old_on_top = false;
    config.layer_drawing = false;
    let vis = |mut config: visualizer::Config, name: &str| {
        config.filepath = PathBuf::from("imgs/fig1-slides/").join(name);
        config
    };

    {
        config.draw_old_on_top = true;
        let mut nw = NW {
            cm: cm.clone(),
            use_gap_cost_heuristic: true,
            exponential_search: true,
            local_doubling: false,
            h: NoCost,
            v: vis(config.clone(), "1_ukkonen"),
        };
        nw.align(a, b);
    }

    {
        let mut a_star = Astar {
            dt: false,
            h: NoCost,
            v: vis(config.clone(), "2_dijkstra"),
        };
        a_star.align(a, b);
    }

    {
        let mut dt = DiagonalTransition::new(
            cm.clone(),
            GapCostHeuristic::Disable,
            NoCost,
            false,
            vis(config.clone(), "3_diagonal-transition"),
        );
        dt.align(a, b);
    }

    {
        config.draw_old_on_top = false;
        let mut dt = DiagonalTransition::new(
            cm.clone(),
            GapCostHeuristic::Disable,
            NoCost,
            true,
            vis(config.clone(), "4_dt-divide-and-conquer"),
        );
        dt.align(a, b);
    }

    {
        let k = 5;
        let h = CSH {
            match_config: MatchConfig::exact(k),
            pruning: Pruning::default(),
            use_gap_cost: false,
            c: PhantomData::<BruteForceContours>::default(),
        };
        let mut a_star = Astar {
            dt: false,
            h,
            v: vis(config.clone(), "5_astar-csh"),
        };
        a_star.align(a, b);
    }

    {
        let k = 5;
        let h = CSH {
            match_config: MatchConfig::exact(k),
            pruning: Pruning::enabled(),
            use_gap_cost: false,
            c: PhantomData::<BruteForceContours>::default(),
        };
        let mut a_star = Astar {
            dt: false,
            h,
            v: vis(config.clone(), "6_astar-csh-pruning"),
        };
        a_star.align(a, b);
    }
}
