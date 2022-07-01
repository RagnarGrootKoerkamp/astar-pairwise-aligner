use super::cigar::Cigar;
use super::diagonal_transition::HistoryCompression;
use super::nw::Path;
use super::{Aligner, Seq, VisualizerT};
use crate::cost_model::*;
use crate::prelude::{to_string, Pos};
use std::cmp::{max, min};
use std::io::stdin;
use std::iter::zip;

pub type Fr = i32;

type Front<const N: usize> = super::front::Front<N, Fr, Fr>;
type Fronts<const N: usize> = super::front::Fronts<N, Fr, Fr>;

/// GapOpen costs can be processed either when entering of leaving the gap.
/// See https://research.curiouscoding.nl/notes/affine-gap-close-cost/.
pub enum GapVariant {
    GapOpen,
    GapClose,
}
use GapVariant::*;

/// The direction to run in.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Forward,
    Backward,
}
use itertools::chain;
use num_traits::abs;
use Direction::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GapCostHeuristic {
    Enable,
    Disable,
}

pub struct DiamondTransition<CostModel, V: VisualizerT> {
    /// The CostModel to use, possibly affine.
    cm: CostModel,
    use_gap_cost_heuristic: GapCostHeuristic,
    gap_variant: GapVariant,
    #[allow(unused)]
    history_compression: HistoryCompression,
    v: V,
    direction: Direction,
    top_buffer: Fr,
    left_buffer: Fr,
    right_buffer: Fr,
}

#[inline]
fn fr_to_coords(d: Fr, f: Fr) -> (Fr, Fr) {
    ((f + d) / 2, (f - d) / 2)
}
#[inline]
fn fr_to_pos(d: Fr, f: Fr) -> Pos {
    Pos(
        ((f + d) / 2) as crate::prelude::I,
        ((f - d) / 2) as crate::prelude::I,
    )
}

/// Given two sequences, a diagonal and point on it, expand it to a FR point.
fn extend_diagonal(direction: Direction, a: Seq, b: Seq, d: Fr, mut fr: Fr) -> Fr {
    let (i, j) = fr_to_coords(d, fr);
    if i as usize >= a.len() || j as usize >= b.len() {
        return fr;
    }

    // TODO: The end check can be avoided by appending `#` and `$` to `a` and `b`.
    match direction {
        Forward => {
            fr += 2 * zip(a[i as usize..].iter(), b[j as usize..].iter())
                .take_while(|(ca, cb)| ca == cb)
                .count() as Fr
        }
        Backward => {
            fr -= 2 * zip(a[..i as usize].iter().rev(), b[..j as usize].iter().rev())
                .take_while(|(ca, cb)| ca == cb)
                .count() as Fr
        }
    };
    fr
}

impl<const N: usize, V: VisualizerT> DiamondTransition<AffineCost<N>, V> {
    pub fn new_variant(
        cm: AffineCost<N>,
        use_gap_cost_heuristic: GapCostHeuristic,
        history_compression: HistoryCompression,
        gap_variant: GapVariant,
        direction: Direction,
        v: V,
    ) -> Self {
        // The maximum cost we look back:
        // max(substitution, indel, affine indel of size 1)
        let top_buffer = max(
            max(
                cm.sub.unwrap_or(0),
                match gap_variant {
                    GapOpen => 0,
                    GapClose => max(cm.max_del_open, cm.max_ins_open),
                },
            ),
            match gap_variant {
                GapOpen => max(cm.max_ins_open_extend, cm.max_del_open_extend),
                GapClose => max(cm.max_ins_extend, cm.max_del_extend),
            },
        ) as Fr;

        let left_buffer = max(
            // substitution, if allowed
            cm.sub
                .unwrap_or(match gap_variant {
                    GapOpen => 0,
                    GapClose => max(cm.max_del_open, cm.max_ins_open),
                })
                .div_ceil(cm.ins.unwrap_or(Cost::MAX)),
            // number of insertions (left moves) done in range of looking one deletion (right move) backwards
            1 + match gap_variant {
                GapOpen => cm.max_del_open_extend,
                GapClose => cm.max_del_extend,
            }
            .div_ceil(cm.min_ins_extend),
        ) as Fr;
        // Idem.
        let right_buffer = max(
            // substitution, if allowed
            cm.sub
                .unwrap_or(match gap_variant {
                    GapOpen => 0,
                    GapClose => max(cm.max_del_open, cm.max_ins_open),
                })
                .div_ceil(cm.del.unwrap_or(Cost::MAX)),
            // number of deletions (right moves) done in range of looking one insertion (left move) backwards
            1 + match gap_variant {
                GapOpen => cm.max_ins_open_extend,
                GapClose => cm.max_ins_extend,
            }
            .div_ceil(cm.min_del_extend),
        ) as Fr;
        Self {
            cm,
            use_gap_cost_heuristic,
            gap_variant,
            v,
            top_buffer,
            left_buffer,
            right_buffer,
            direction,
            history_compression,
        }
    }

    pub fn new(
        cm: AffineCost<N>,
        use_gap_cost_heuristic: GapCostHeuristic,
        history_compression: HistoryCompression,
        v: V,
    ) -> Self {
        Self::new_variant(
            cm,
            use_gap_cost_heuristic,
            history_compression,
            GapOpen,
            Forward,
            v,
        )
    }

    fn extend(&mut self, front: &mut Front<N>, a: Seq, b: Seq, d: Fr) -> bool {
        // let tmp = front.range().clone();
        // println!("Extend range: {tmp:?}\n");
        let fr = &mut front.m_mut()[d];
        println!("Extend1: {d}\t{fr}");
        if *fr < 0 {
            return false;
        }
        let fr_old = *fr;
        *fr = match self.direction {
            Forward => extend_diagonal(self.direction, a, b, d, *fr),
            Backward => extend_diagonal(
                self.direction,
                a,
                b,
                a.len() as Fr - b.len() as Fr - d,
                a.len() as Fr + b.len() as Fr - *fr,
            ),
        };
        let mut p = fr_to_pos(d, fr_old);
        for _ in fr_old..*fr {
            p = p.add_diagonal(1);
            self.v.expand(p);
        }
        println!("Extend3: {d}\t{fr}");

        println!(
            "Extend2: {}\t{}\t{}",
            front.range().contains(&(a.len() as Fr - b.len() as Fr)),
            front.m_mut()[a.len() as Fr - b.len() as Fr],
            (a.len() + b.len()) as Fr
        );
        if d == (a.len() as Fr - b.len() as Fr) && front.m_mut()[d] >= (a.len() + b.len()) as Fr {
            return true;
        }
        false
    }

    fn get_front<'a>(
        &self,
        cost: Cost,
        d: Fr,
        next: &'a Front<N>,
        prev: &'a [Front<N>],
        d0: Fr,
        s: Cost,
    ) -> &'a Front<N> {
        // println!("AAA {s}\t{cost}\t{d}\t");
        // println!(
        //     "{}",
        //     (s as Fr - cost as Fr
        //         + if d0 != 0 && d0 != d { d } else { 0 }
        //         + if (d < d0 && d0 < 0) || (d > d0 && d0 > 0) {
        //             d0 - d
        //         } else {
        //             0
        //         }) as Fr
        // );
        if 2 + s as Fr - cost as Fr == prev.len() as Fr {
            return next;
        }
        if 2 + s as Fr - cost as Fr > prev.len() as Fr {
            return &prev[0];
        }
        &prev[max(2 + s as Fr - cost as Fr, 0) as usize]
    }

    fn next_front(
        &mut self,
        a: Seq,
        b: Seq,
        prev: &[Front<N>],
        next: &mut Front<N>,
        mut s: Cost,
    ) -> bool {
        s -= 1;
        // for index1 in 0..prev.len() {
        //     for index2 in prev[index1].range().clone() {
        //         print!("{} ", prev[index1].m()[index2]);
        //     }
        //     print!("\n");
        // }

        println!("\nNEXT FRONT START {}\n\n", s);
        // println!("prev_len is {}", prev.len());

        let d0 = a.len() as Fr - b.len() as Fr;
        let sgn = |d: Fr, pos: Fr| -> Fr {
            //pos: +1 - - diagonal to left (d + 1); 0 - same diagonal; -1 - diagonal to right (d - 1)
            if d0 == 0 {
                return 0;
            }
            if d == d0 {
                if pos == 0 {
                    return 0;
                } else {
                    return -1;
                }
            }
            if d > d0 {
                if pos == 0 {
                    return 0;
                } else {
                    return pos;
                }
            }
            if d < d0 {
                if pos == 0 {
                    return 0;
                } else {
                    return -pos;
                }
            }
            return 0;
        };
        // Get the front `cost` before the last one.
        // let get_front = |cost, d: Fr, next: &Front<N>, prev: &[Front<N>]| {
        //     println!("AAA {s}\t{cost}\t{d}\t");
        //     println!(
        //         "{}",
        //         (s as Fr - cost as Fr
        //             + if d0 != 0 && d0 != d { d } else { 0 }
        //             + if (d < d0 && d0 < 0) || (d > d0 && d0 > 0) {
        //                 d0 - d
        //             } else {
        //                 0
        //             }) as Fr
        //     );
        //     if 2 + s as Fr - cost as Fr == prev.len() as Fr {
        //         return next;
        //     }
        //     if 2 + s as Fr - cost as Fr > prev.len() as Fr {
        //         return &prev[0];
        //     }
        //     &prev[max(2 + s as Fr - cost as Fr, 0) as usize]
        // };
        // let get_front = |cost| &prev[prev.len() - cost as usize];

        if s == 0 {
            next.m_mut()[0] = 0;
        }

        match self.gap_variant {
            GapOpen => {
                // Loop over the entire dmin..=dmax range.
                // The boundaries are buffered so no boundary checks are needed.
                // TODO: Vectorize this loop, or at least verify the compiler does this.
                // TODO: Loop over a positive range that does not need additional shifting?
                // for d in next.range().clone() {
                for d in chain(*next.range().start()..d0, (d0..=*next.range().end()).rev()) {
                    println!("\na == {}\nb == {}\n\n", to_string(a), to_string(b));
                    // The new value of next.m[d].
                    let mut f = next.m_mut()[d];
                    // println!("f = {f}");
                    // Affine layers
                    for layer_idx in 0..N {
                        let cm = &self.cm.affine[layer_idx];
                        let affine_f = match cm.affine_type {
                            InsertLayer => max(
                                // Gap open
                                self.get_front(
                                    ((cm.open + cm.extend) as Fr - sgn(d + 1, 1)) as Cost,
                                    d + 1,
                                    next,
                                    prev,
                                    d0,
                                    s,
                                )
                                .m()[d + 1]
                                    + 1,
                                // Gap extend
                                self.get_front(
                                    (cm.extend as Fr - sgn(d + 1, 1)) as Cost,
                                    d + 1,
                                    next,
                                    prev,
                                    d0,
                                    s,
                                )
                                .affine(layer_idx)[d + 1]
                                    + 1,
                            ),
                            DeleteLayer => max(
                                // Gap open
                                self.get_front(
                                    ((cm.open + cm.extend) as Fr - sgn(d - 1, -1)) as Cost,
                                    d - 1,
                                    next,
                                    prev,
                                    d0,
                                    s,
                                )
                                .m()[d - 1]
                                    + 1,
                                // Gap extend
                                self.get_front(
                                    (cm.extend as Fr - sgn(d - 1, -1)) as Cost,
                                    d - 1,
                                    next,
                                    prev,
                                    d0,
                                    s,
                                )
                                .affine(layer_idx)[d - 1]
                                    + 1,
                            ),
                            _ => todo!(),
                        };
                        next.affine_mut(layer_idx)[d] = affine_f;
                        // Gap close
                        f = max(f, affine_f);
                    }
                    // Substitution
                    if let Some(cost) = self.cm.sub {
                        println!("Substitution {cost}!\n");
                        println!("d == {}\n", d);
                        println!("f = {f}");
                        f = max(f, self.get_front(cost, d, next, prev, d0, s).m()[d] + 2);
                        println!("f = {f}");
                    }
                    // Insertion
                    if let Some(cost) = self.cm.ins {
                        println!("Insertion {cost}!\n");
                        println!("d == {}\n", d);
                        println!("f = {f}");
                        f = max(
                            f,
                            self.get_front(
                                (cost as Fr - sgn(d + 1, 1)) as Cost,
                                d + 1,
                                next,
                                prev,
                                d0,
                                s,
                            )
                            .m()[d + 1]
                                + 1,
                        );
                        println!("f = {f}");
                    }
                    // Deletion
                    if let Some(cost) = self.cm.del {
                        println!("Deletion {cost}!\n");
                        println!("d == {}\n", d);
                        println!("f = {f}");
                        f = max(
                            f,
                            self.get_front(
                                (cost as Fr - sgn(d - 1, -1)) as Cost,
                                d - 1,
                                next,
                                prev,
                                d0,
                                s,
                            )
                            .m()[d - 1]
                                + 1,
                        );
                        println!("f = {f}");
                    }
                    next.m_mut()[d] = f;

                    if f >= 0 {
                        self.v.expand(fr_to_pos(d, f));
                    }

                    if self.extend(next, a, b, d) {
                        return true;
                    }
                }
                return false;
                // Extend all points in the m layer and check if we're done.
            }
            // TODO: the 'graph' related code should be elsewhere, and this should just follow the graph.
            GapClose => {
                // See https://research.curiouscoding.nl/notes/affine-gap-close-cost/.
                for d in chain(*next.range().start()..d0, (d0..=*next.range().end()).rev()) {
                    // The new value of next.m[d].
                    let mut f = next.m_mut()[d];
                    // Substitution
                    if let Some(cost) = self.cm.sub {
                        f = max(f, self.get_front(cost, d, next, prev, d0, s).m()[d] + 2);
                    }
                    // Insertion
                    if let Some(cost) = self.cm.ins {
                        f = max(
                            f,
                            self.get_front(cost, d + 1, next, prev, d0, s).m()[d + 1] + 1,
                        );
                    }
                    // Deletion
                    if let Some(cost) = self.cm.del {
                        f = max(
                            f,
                            self.get_front(cost, d - 1, next, prev, d0, s).m()[d - 1] + 1,
                        );
                    }
                    // Affine layers: Gap close
                    for idx in 0..N {
                        let cm = &self.cm.affine[idx];
                        match cm.affine_type {
                            InsertLayer | DeleteLayer => {
                                // Gap close
                                f = max(
                                    f,
                                    self.get_front(cm.open + cm.extend, d, next, prev, d0, s)
                                        .m()[d],
                                )
                            }
                            _ => todo!(),
                        };
                    }
                    next.m_mut()[d] = f;

                    self.v.expand(fr_to_pos(d, f));
                    if self.extend(next, a, b, d) {
                        return true;
                    }
                }
                // Extend all points in the m layer and check if we're done.

                for d in next.range().clone() {
                    // Affine layers: Gap open/extend
                    for idx in 0..N {
                        let cm = &self.cm.affine[idx];
                        next.affine_mut(idx)[d] = match cm.affine_type {
                            // max(Gap open, Gap extend)
                            InsertLayer => max(
                                next.m()[d + 1] + 1,
                                self.get_front(cm.extend, d + 1, next, prev, d0, s)
                                    .affine(idx)[d + 1]
                                    + 1,
                            ),
                            // max(Gap open, Gap extend)
                            DeleteLayer => max(
                                next.m()[d - 1] + 1,
                                self.get_front(cm.extend, d - 1, next, prev, d0, s)
                                    .affine(idx)[d - 1]
                                    + 1,
                            ),
                            _ => todo!(),
                        };
                    }
                    // FIXME
                    //v.expand(fr_to_pos(d, f));
                }
                false
            }
        }
    }

    // Returns None when the sequences are equal.
    fn init_fronts(&mut self, a: Seq, b: Seq) -> Option<Fronts<N>> {
        let fronts = Fronts::new(
            Fr::MIN,
            // We only create a front for the s=0 layer.
            0..=0,
            // The range of the s=0 front is 0..=0.
            |_| -1000..=1000,
            // Additionally, we have `top_buffer` fronts before the current front.
            self.top_buffer,
            0,
            self.left_buffer,
            self.right_buffer,
        );

        // let f = extend_diagonal(self.direction, a, b, 0, 0);
        // fronts[0].m_mut()[0] = f;
        // if f >= (a.len() + b.len()) as Fr {
        //     return None;
        // }
        Some(fronts)
    }
}

impl<const N: usize, V: VisualizerT> Aligner for DiamondTransition<AffineCost<N>, V> {
    type CostModel = AffineCost<N>;

    fn cost_model(&self) -> &Self::CostModel {
        &self.cm
    }

    /// The cost-only version uses linear memory.
    ///
    /// In particular, the number of fronts is max(sub, ins, del)+1.
    fn cost_for_bounded_dist(&mut self, a: Seq, b: Seq, s_bound: Option<Cost>) -> Option<Cost> {
        // println!("Cost for bounded dist");
        if let Some((cost, path, cigar)) = self.align_for_bounded_dist(a, b, None) {
            return Some(cost);
        } else {
            return None;
        }
        // I haven't change this function; it uses old algorithm
        //     let Some(mut fronts) = self.init_fronts(a, b) else {
        //     return Some(0);
        // };

        //     let mut s = 0;
        //     loop {
        //         if let Some(s_bound) = s_bound && s >= s_bound {
        //         return None;
        //     }

        //         s += 1;

        //         // Rotate all fronts back by one, so that we can fill the new last layer.
        //         fronts.fronts.rotate_left(1);
        //         let (next, fronts) = fronts.fronts.split_last_mut().unwrap();

        //         next.reset(
        //             Fr::MIN,
        //             0..=0,
        //             // self.d_range(a, b, s, s_bound),
        //             self.left_buffer,
        //             self.right_buffer,
        //         );
        //         if self.next_front(a, b, fronts, next, s) {
        //             return Some(s);
        //         }
        //     }
    }

    fn align_for_bounded_dist(
        &mut self,
        a: Seq,
        b: Seq,
        s_bound: Option<Cost>,
    ) -> Option<(Cost, Path, Cigar)> {
        // println!("Align for bounded dist");
        let Some(mut fronts) = self.init_fronts(a, b) else {
        return Some((0, vec![], Cigar::default()));
    };

        println!("\na == {}\nb == {}\n\n", to_string(a), to_string(b));

        let d0 = a.len() as Fr - b.len() as Fr;

        let mut s = 1;
        loop {
            if let Some(s_bound) = s_bound && s > s_bound {
            return None;
        }
            println!("s == {s}");

            // We can not initialize all layers directly at the start, since we do not know the final distance s.
            let mut next = Front::new(
                Fr::MIN,
                //self.d_range(a, b, s, s_bound),
                (min(d0, 0) - ((s - 1) / if d0 == 0 { 1 } else { 2 }) as Fr)
                    ..=(max(d0, 0) + ((s - 1) / if d0 == 0 { 1 } else { 2 }) as Fr),
                self.left_buffer,
                self.right_buffer,
            );
            if self.next_front(a, b, &fronts.fronts, &mut next, s) {
                // FIXME: Reconstruct path.
                // let path2 = self.get_path(&fronts, a, b, s);
                // let cigar2 = self.get_cigar(&fronts, a, b, s);
                // println!("RETURN {}", d0.abs() as u32 + s - 1);
                return Some((d0.abs() as u32 + s - 1, vec![], Cigar::default()));
            }

            fronts.fronts.push(next);
            s += 1;

            // let mut str = String::from("Hi");
            // stdin()
            //     .read_line(&mut str)
            //     .expect("Did not enter a correct string");
            println!("Loop iteration end {s}");
        }
    }
}
