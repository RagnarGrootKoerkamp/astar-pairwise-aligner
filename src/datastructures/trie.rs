use bio::alphabets::{Alphabet, RankTransform};

use crate::prelude::*;

pub type State = u32;
pub type Data = u32;
/// A potentially smaller seed length datatype to save stack space.
pub type MatchLen = u8;

pub struct TrieNode {
    // Child indices are always positive, so 0 indicates empty.
    children: [State; 4],
    data: Vec<u32>,
}

impl Default for TrieNode {
    fn default() -> Self {
        Self {
            children: [State::MAX; 4],
            data: Vec::default(),
        }
    }
}

pub struct Trie {
    states: Vec<TrieNode>,
    pub transform: RankTransform,
}

impl Trie {
    pub fn push(self: &mut Trie, word: &[u8], data: Data) {
        let mut state = 0 as State;
        for c in word {
            state = self.force_step(state, *c);
        }
        self.states[state as usize].data.push(data);
    }

    pub fn new<'a>(words: impl IntoIterator<Item = (&'a [u8], Data)>, alph: &Alphabet) -> Self {
        assert!(
            alph.len() <= 4,
            "Trie can only be used for DNA alphabet for now."
        );
        let mut this = Trie {
            states: Vec::default(),
            transform: RankTransform::new(alph),
        };
        this.states.push(TrieNode::default());
        for (word, data) in words {
            this.push(word, data);
        }
        this
    }

    fn force_step(&mut self, state: State, c: u8) -> State {
        let len = self.states.len();
        match &mut self.states[state as usize].children[self.transform.get(c) as usize] {
            x if *x < State::MAX => *x,
            x => {
                *x = len as State;
                let x = *x;
                self.states.push(TrieNode::default());
                x
            }
        }
    }

    #[inline]
    fn emit_subtree<F: FnMut(Data, MatchLen, MatchCost)>(
        &self,
        cost: MatchCost,
        depth: MatchLen,
        state: State,
        f: &mut F,
    ) {
        for d in &self.states[state as usize].data {
            f(*d, depth, cost);
        }
        for state in self.states[state as usize].children {
            if state != State::MAX {
                self.emit_subtree(cost, depth, state, f);
            }
        }
    }

    /// Stack based implementation of the DFS version above.
    #[inline]
    pub fn matches<F: FnMut(Data, MatchLen, MatchCost)>(
        &self,
        seed: &[u8],
        max_cost: MatchCost,
        //cost_model: CostModel,
        mut f: F,
    ) {
        let cost_model = crate::cost_model::LinearCost::new_unit();
        struct QueueElement {
            /// Current state in tree
            state: State,
            /// Position in seed
            i: MatchLen,
            /// Depth in tree
            j: MatchLen,
            /// Cost of match so far
            cost: MatchCost,
            /// True if ending with an insert.
            last_is_insert: bool,
        }

        // TODO: BFS vs DFS vs Dijkstra?
        let mut queue = vec![QueueElement {
            state: 0,
            i: 0,
            j: 0,
            cost: 0,
            last_is_insert: false,
        }];

        while let Some(QueueElement {
            state,
            i,
            j,
            cost,
            last_is_insert,
        }) = queue.pop()
        {
            //println!("Trie state {state} max cost {max_cost} cur cost {cost} at depth {depth} remaining {remaining_seed:?}");
            // No more matches

            if (i as usize) == seed.len() {
                if SKIP_INEXACT_INSERT_START_END && last_is_insert {
                    continue;
                }
                // If the remaining part is empty, emit this subtree.
                self.emit_subtree(cost, j, state, &mut f);
            } else {
                // Match or substitute a char.
                let matching_index = self.transform.get(seed[i as usize]) as usize;
                for (ci, state) in self.states[state as usize].children.iter().enumerate() {
                    if *state == State::MAX {
                        continue;
                    }
                    let mismatch_cost = if ci == matching_index {
                        0
                    } else {
                        let Some(x) = cost_model.sub() else {continue;};
                        x
                    };
                    if cost + mismatch_cost as MatchCost > max_cost {
                        continue;
                    }
                    queue.push(QueueElement {
                        state: *state,
                        i: i + 1,
                        j: j + 1,
                        cost: cost + mismatch_cost as MatchCost,
                        last_is_insert: false,
                    });
                }

                // Delete a char: the character in the seed is ignored, and we remain at the same depth.
                if let Some(del) = cost_model.del() {
                    if cost + del as MatchCost <= max_cost {
                        queue.push(QueueElement {
                            state,
                            i: i + 1,
                            j,
                            cost: cost + del as MatchCost,
                            last_is_insert: false,
                        });
                    }
                }
            }

            // Insert a char: No character from the seed is needed, but we increase the depth.
            // NOTE: We never insert the next character in the string, as we could directly match that instead.
            if SKIP_INEXACT_INSERT_START_END && state == 0 {
                continue;
            }
            if let Some(ins) = cost_model.ins() {
                let matching_index = seed
                    .get(i as usize)
                    .map(|c| self.transform.get(*c) as usize);
                for (ci, state) in self.states[state as usize].children.iter().enumerate() {
                    if *state == State::MAX {
                        continue;
                    }
                    if Some(ci) == matching_index {
                        continue;
                    }
                    if cost + ins as MatchCost > max_cost {
                        continue;
                    }
                    queue.push(QueueElement {
                        state: *state,
                        i,
                        j: j + 1,
                        cost: cost + ins as MatchCost,
                        last_is_insert: true,
                    });
                }
            }
        }
    }
}
