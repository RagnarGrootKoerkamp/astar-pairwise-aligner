use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize)]
#[clap(next_help_heading = "Algorithm")]
pub struct AlgorithmArgs {
    // #[clap(short, long, default_value_t, value_enum, display_order = 10)]
    // pub algorithm: Algorithm,
    /// Use diagonal-transition based A*.
    #[clap(long, hide_short_help = true)]
    pub dt: bool,

    /// Use exponential search in NW algorithm (like edlib).
    #[clap(long, hide_short_help = true)]
    pub exp_search: bool,

    /// Use local doubling in NW/DT.
    #[clap(long, hide_short_help = true)]
    pub local_doubling: bool,

    /// Use divide and conquer for diagonal transition (like BiWFA).
    #[clap(long, hide_short_help = true)]
    pub dc: bool,
}

// /// Convert to a title string for the visualizer.
// impl ToString for AlgorithmArgs {
//     fn to_string(&self) -> String {
//         match Algorithm::Astar {
//             // Algorithm::NW => {
//             //     let mut s = "Needleman-Wunsch".to_string();
//             //     if self.exp_search {
//             //         s += " + Doubling";
//             //     }
//             //     if self.local_doubling {
//             //         s += " + Local Doubling";
//             //     }
//             //     s
//             // }
//             // Algorithm::DT => {
//             //     let mut s = "Diagonal Transition".to_string();
//             //     if self.dc {
//             //         s += " + Divide & Conquer";
//             //     }
//             //     if self.local_doubling {
//             //         s += " + Local Doubling";
//             //     }
//             //     s
//             // }
//             Algorithm::Astar => {
//                 let mut s = "A*".to_string();
//                 if self.dt {
//                     s += " + Diagonal Transition";
//                 }
//                 s
//             } // Algorithm::TripleAccel => {
//               //     if self.exp_search {
//               //         "Needleman-Wunsch + Doubling (triple-accel)".into()
//               //     } else {
//               //         "Needleman-Wunsch (triple-accel)".into()
//               //     }
//               // }
//         }
//     }
// }

// pub fn comment(alg: &AlgorithmArgs, h: &HeuristicArgs) -> Option<String> {
//     match Algorithm::Astar {
//         // Algorithm::NW => {
//         //     assert!(
//         //         !(alg.exp_search && alg.local_doubling),
//         //         "Cannot not do both exponential search and local doubling at the same time."
//         //     );

//         //     if !alg.exp_search && !alg.local_doubling {
//         //         Some("Visit all states ordered by i".into())
//         //     } else if alg.exp_search {
//         //         match h.heuristic {
//         //             HeuristicType::None => {
//         //                 if !h.gap_cost {
//         //                     Some("Visit Gap(s, u) ≤ Fmax ordered by i".into())
//         //                 } else {
//         //                     Some("Visit Gap(s, u) + Gap(u, t) ≤ Fmax ordered by i".into())
//         //                 }
//         //             }
//         //             HeuristicType::Zero => Some("Visit g ≤ Fmax ordered by i".into()),
//         //             HeuristicType::Gap => Some("Visit g + Gap(u, t) ≤ Fmax ordered by i".into()),
//         //             HeuristicType::SH => Some("Visit g + SH(u) ≤ Fmax ordered by i".into()),
//         //             HeuristicType::CSH => Some("Visit g + CSH(u) ≤ Fmax ordered by i".into()),
//         //         }
//         //     } else {
//         //         assert!(alg.local_doubling);
//         //         match h.heuristic {
//         //             HeuristicType::None => panic!("Local doubling requires a heuristic!"),
//         //             HeuristicType::Zero => Some("Visit g ≤ Fmax(i) ordered by i".into()),
//         //             HeuristicType::Gap => Some("Visit g + Gap(u, t) ≤ Fmax(i) ordered by i".into()),
//         //             HeuristicType::SH => Some("Visit g + SH(u) ≤ Fmax(i) ordered by i".into()),
//         //             HeuristicType::CSH => Some("Visit g + CSH(u) ≤ Fmax(i) ordered by i".into()),
//         //         }
//         //     }
//         // }
//         // Algorithm::DT => {
//         //     if !alg.local_doubling {
//         //         Some("Visit fr. states by g".into())
//         //     } else {
//         //         assert!(alg.local_doubling);
//         //         match h.heuristic {
//         //             HeuristicType::None => panic!("Local doubling requires a heuristic!"),
//         //             HeuristicType::Zero => Some("Visit fr. states g ≤ Fmax(i) ordered by g".into()),
//         //             HeuristicType::Gap => {
//         //                 Some("Visit fr. states g + Gap(u, t) ≤ Fmax(i) ordered by g".into())
//         //             }
//         //             HeuristicType::SH => {
//         //                 Some("Visit fr. states g + SH(u) ≤ Fmax(i) ordered by g".into())
//         //             }
//         //             HeuristicType::CSH => {
//         //                 Some("Visit fr. states g + CSH(u) ≤ Fmax(i) ordered by g".into())
//         //             }
//         //         }
//         //     }
//         // }
//         Algorithm::Astar => {
//             if !alg.dt {
//                 match h.heuristic {
//                     HeuristicType::None | HeuristicType::Zero => {
//                         Some("A* without heuristic is simply Dijkstra".into())
//                     }
//                     HeuristicType::Gap => Some("Visit states ordered by f = g + Gap(u, t)".into()),
//                     HeuristicType::SH => Some("Visit states ordered by f = g + SH(u)".into()),
//                     HeuristicType::CSH => Some("Visit states ordered by f = g + CSH(u)".into()),
//                 }
//             } else {
//                 match h.heuristic {
//                     HeuristicType::None | HeuristicType::Zero => {
//                         Some("A* without heuristic is simply Dijkstra".into())
//                     }
//                     HeuristicType::Gap => {
//                         Some("Visit fr. states ordered by f = g + Gap(u, t)".into())
//                     }
//                     HeuristicType::SH => Some("Visit fr. states ordered by f = g + SH(u)".into()),
//                     HeuristicType::CSH => Some("Visit fr. states ordered by f = g + CSH(u)".into()),
//                 }
//             }
//         }
//     }
// }
