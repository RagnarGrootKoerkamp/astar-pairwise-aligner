use bio::alphabets::Alphabet;

use crate::{alignment_graph, astar::astar};

use super::{cigar::Cigar, edit_graph::State};
use crate::{
    astar_dt::astar_dt, cost_model::LinearCost, heuristic::Heuristic, prelude::Pos,
    visualizer::VisualizerT,
};

use super::Aligner;

pub struct AStar<V: VisualizerT, H: Heuristic> {
    pub greedy_edge_matching: bool,
    pub diagonal_transition: bool,

    /// The heuristic to use.
    pub h: H,

    /// The visualizer to use.
    pub v: V,
}

impl<V: VisualizerT, H: Heuristic> std::fmt::Debug for AStar<V, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AStar")
            .field("greedy_edge_matching", &self.greedy_edge_matching)
            .field("diagonal_transition", &self.diagonal_transition)
            .field("h", &self.h)
            .finish()
    }
}

impl<V: VisualizerT, H: Heuristic> Aligner for AStar<V, H> {
    type CostModel = LinearCost;
    type Fronts = usize;

    type State = State;

    fn cost_model(&self) -> &Self::CostModel {
        todo!()
    }

    fn parent(
        &self,
        _a: super::Seq,
        _b: super::Seq,
        _fronts: &Self::Fronts,
        _st: Self::State,
        _direction: super::diagonal_transition::Direction,
    ) -> Option<(Self::State, super::edit_graph::CigarOps)> {
        unimplemented!("Sorry, I have no idea what this function does. My only task is to make this thing work at any cost.");
    }

    fn cost(&mut self, a: super::Seq, b: super::Seq) -> crate::cost_model::Cost {
        return self.align(a, b).0;
    }

    fn align(
        &mut self,
        a: super::Seq,
        b: super::Seq,
    ) -> (crate::cost_model::Cost, super::cigar::Cigar) {
        // Instantiate the heuristic.
        let ref mut h = self.h.build(a, b, &Alphabet::new(b"ACTG"));

        // Run A* with heuristic.
        // TODO: Make the greedy_matching bool a parameter in a struct with A* options.
        let graph = alignment_graph::EditGraph::new(a, b, self.greedy_edge_matching);
        let (distance_and_path, _) = if self.diagonal_transition {
            astar_dt(&graph, h)
        } else {
            astar(&graph, h, &mut self.v)
        };
        let (distance, path) = distance_and_path.unwrap_or_default();

        let path: Vec<Pos> = path.into_iter().collect();
        return (distance, Cigar::from_path(a, b, &path));
    }

    fn cost_for_bounded_dist(
        &mut self,
        _a: super::Seq,
        _b: super::Seq,
        _s_bound: Option<crate::cost_model::Cost>,
    ) -> Option<crate::cost_model::Cost> {
        unimplemented!("Astar doesn't support it");
    }

    fn align_for_bounded_dist(
        &mut self,
        _a: super::Seq,
        _b: super::Seq,
        _s_bound: Option<crate::cost_model::Cost>,
    ) -> Option<(crate::cost_model::Cost, super::cigar::Cigar)> {
        unimplemented!("Astar doesn't support it");
    }
}
