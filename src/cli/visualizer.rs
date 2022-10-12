use clap::{ArgMatches, Parser};

use crate::{
    prelude::Seq,
    visualizer::{NoVisualizer, VisualizerStyle, VisualizerT, When},
};

#[derive(Parser)]
#[clap(help_heading = "VISUALIZER")]
pub struct VisualizerArgs {
    /// Run the interactive visualizer. See --help for controls. [default: all]
    ///
    /// x: exit
    /// p: pause / unpaues
    /// esc, space: next frame
    /// f: faster
    /// s: slower
    /// q: jump to last frame, or exit when already on last frame
    #[clap(
        short = 'v',
        long,
        display_order = 1,
        min_values = 0,
        value_enum,
        value_name = "WHEN"
    )]
    pub visualize: Option<When>,

    /// Visualizer style.
    #[clap(long, default_value_t, value_enum, display_order = 2)]
    pub style: VisualizerStyle,

    /// Which frames to save.
    #[clap(
        long,
        display_order = 3,
        min_values = 0,
        value_enum,
        value_name = "WHEN"
    )]
    pub save: Option<When>,

    /// Where to save. Implies --save [last].
    #[clap(long, display_order = 4, value_name = "PATH")]
    pub save_path: Option<String>,

    /// The size in pixels of each cell.
    /// By default, chosen to give a canvas of height 500.
    #[clap(long, display_order = 10, hide_short_help = true)]
    pub cell_size: Option<u32>,

    /// Number of states per cell.
    /// By default, chosen to give a canvas of height 500.
    #[clap(long, display_order = 10, hide_short_help = true)]
    pub downscaler: Option<u32>,

    /// When set, draw newer expanded states on top. Useful for divide & conquer approaches.
    #[clap(long, display_order = 10, hide_short_help = true)]
    pub new_on_top: bool,

    /// Enable drawing the tree.
    #[clap(long, display_order = 10, hide_short_help = true)]
    pub draw_tree: bool,

    /// Disable drawing the tree.
    #[clap(long, display_order = 10, hide_short_help = true)]
    pub no_draw_tree: bool,
}

pub trait VisualizerRunner {
    type R;
    fn call<V: VisualizerT>(&self, v: V) -> Self::R;
}

impl VisualizerArgs {
    // pass matches as <Cli as clap::CommandFactory>::command().get_matches()
    #[cfg(not(feature = "sdl2"))]
    pub fn run_on_visualizer<F: VisualizerRunner>(
        &self,
        _a: Seq,
        _b: Seq,
        matches: ArgMatches,
        f: F,
    ) -> F::R {
        if matches.contains_id("visualize")
            || matches.contains_id("save")
            || matches.contains_id("save_path")
        {
            eprintln!("The sdl2 feature must be enabled to use visualizations.");
            panic!("The sdl2 feature must be enabled to use visualizations.");
        }

        f.call(NoVisualizer)
    }

    #[cfg(feature = "sdl2")]
    pub fn run_on_visualizer<F: VisualizerRunner>(
        &self,
        a: Seq,
        b: Seq,
        matches: ArgMatches,
        f: F,
    ) -> F::R {
        use sdl2::pixels::Color;

        use crate::visualizer::{Config, Visualizer};

        let draw = if matches.contains_id("visualize") {
            self.visualize.clone().unwrap_or(When::All)
        } else {
            When::None
        };
        let save = if matches.contains_id("save") {
            self.save.clone().unwrap_or(When::Last)
        } else if self.save_path.is_some() {
            When::Last
        } else {
            When::None
        };
        if draw == When::None && save == When::None {
            return f.call(NoVisualizer);
        }

        // Get the default config for the style.
        let mut config = Config::new(self.style);
        config.draw = draw;
        config.save = save;
        if config.save != When::None {
            config.save_last = true;
            // In this case, the save_last above is sufficient.
            if config.save == When::Last {
                config.save = When::None;
            }
            config.filepath = self
                .save_path
                .clone()
                .expect("--save-path must be set when --save is set");
        }

        // Apply CLI flag customizations to the style.
        config.cell_size = self.cell_size.unwrap_or(0);
        config.downscaler = self.downscaler.unwrap_or(0);

        config.draw_old_on_top = !self.new_on_top;
        if self.draw_tree {
            config.style.tree = Some(Color::BLACK);
        }
        if self.no_draw_tree {
            config.style.tree = None;
        }

        f.call(Visualizer::new(config, a, b))
    }
}
