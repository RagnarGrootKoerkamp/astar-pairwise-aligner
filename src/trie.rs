use bio::alphabets::{Alphabet, RankTransform};

use crate::prelude::USE_TRIE_STACK;

pub type State = u32;
pub type Data = u32;
/// A smaller cost datatype to save stack space.
pub type MatchCost = u8;
/// A smaller seed length datatype to save stack space.
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

    #[inline]
    fn matches_from<F: FnMut(Data, MatchLen, MatchCost)>(
        &self,
        remaining_seed: &[u8],
        max_cost: MatchCost,
        cost: MatchCost,
        depth: MatchLen,
        state: State,
        f: &mut F,
    ) {
        //println!("Trie state {state} max cost {max_cost} cur cost {cost} at depth {depth} remaining {remaining_seed:?}");
        // No more matches
        if state == State::MAX {
            return;
        }

        // Stop here: Walk the entire remaining subtree.
        if let Some((c, remaining_seed)) = remaining_seed.split_first() {
            // Match or substitute a char.
            let matching_index = self.transform.get(*c) as usize;
            for (i, state) in self.states[state as usize].children.iter().enumerate() {
                // TODO: Replace with actual costs.
                let mismatch_cost = if i == matching_index { 0 } else { 1 };
                if cost + mismatch_cost > max_cost {
                    continue;
                }
                self.matches_from(
                    remaining_seed,
                    max_cost,
                    cost + mismatch_cost,
                    depth + 1,
                    *state,
                    f,
                );
            }

            // Delete a char: the character in the seed is ignored, and we remain at the same depth.
            // TODO: Replace with actual costs.
            let deletion_cost = 1;
            if cost + deletion_cost > max_cost {
            } else {
                self.matches_from(
                    remaining_seed,
                    max_cost,
                    cost + deletion_cost,
                    depth,
                    state,
                    f,
                );
            }
        } else {
            // If the remaining part is empty, emit this subtree.
            self.emit_subtree(cost, depth, state, f);
        }

        // Insert a char: No character from the seed is needed, but we increase the depth.
        // NOTE: We never insert the next character in the string, as we could directly match that instead.
        // TODO: Replace with actual costs.
        let insertion_cost = 1;
        let matching_index = remaining_seed
            .first()
            .map(|c| self.transform.get(*c) as usize);
        for (i, state) in self.states[state as usize].children.iter().enumerate() {
            if Some(i) == matching_index {
                continue;
            }
            if cost + insertion_cost > max_cost {
                continue;
            }
            self.matches_from(
                remaining_seed,
                max_cost,
                cost + insertion_cost,
                depth + 1,
                *state,
                f,
            );
        }
    }

    /// Stack based implementation of the DFS version above.
    #[inline]
    fn matches_from_stack<F: FnMut(Data, MatchLen, MatchCost)>(
        &self,
        seed: &[u8],
        max_cost: MatchCost,
        f: &mut F,
    ) {
        struct QueueElement {
            /// Current state in tree
            state: State,
            /// Position in seed
            i: MatchLen,
            /// Depth in tree
            j: MatchLen,
            /// Cost of match so far
            cost: MatchCost,
        }

        // TODO: BFS vs DFS vs Dijkstra?
        let mut queue = vec![QueueElement {
            state: 0,
            i: 0,
            j: 0,
            cost: 0,
        }];

        while let Some(QueueElement { state, i, j, cost }) = queue.pop() {
            //println!("Trie state {state} max cost {max_cost} cur cost {cost} at depth {depth} remaining {remaining_seed:?}");
            // No more matches

            if (i as usize) == seed.len() {
                // If the remaining part is empty, emit this subtree.
                self.emit_subtree(cost, j, state, f);
            } else {
                // Match or substitute a char.
                let matching_index = self.transform.get(seed[i as usize]) as usize;
                for (ci, state) in self.states[state as usize].children.iter().enumerate() {
                    if *state == State::MAX {
                        continue;
                    }
                    // TODO: Replace with actual costs.
                    let mismatch_cost = if ci == matching_index { 0 } else { 1 };
                    if cost + mismatch_cost > max_cost {
                        continue;
                    }
                    queue.push(QueueElement {
                        state: *state,
                        i: i + 1,
                        j: j + 1,
                        cost: cost + mismatch_cost,
                    });
                }

                // Delete a char: the character in the seed is ignored, and we remain at the same depth.
                // TODO: Replace with actual costs.
                let deletion_cost = 1;
                if cost + deletion_cost <= max_cost {
                    queue.push(QueueElement {
                        state,
                        i: i + 1,
                        j,
                        cost: cost + deletion_cost,
                    });
                }
            }

            // Insert a char: No character from the seed is needed, but we increase the depth.
            // NOTE: We never insert the next character in the string, as we could directly match that instead.
            // TODO: Replace with actual costs.
            let insertion_cost = 1;
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
                if cost + insertion_cost > max_cost {
                    continue;
                }
                queue.push(QueueElement {
                    state: *state,
                    i,
                    j: j + 1,
                    cost: cost + insertion_cost,
                });
            }
        }
    }

    /// Finds all matches within the given edit distance, and call `f` for each of them.
    pub fn matches<F: FnMut(Data, MatchLen, MatchCost)>(
        &self,
        seed: &[u8],
        max_cost: MatchCost,
        mut f: F,
    ) {
        if USE_TRIE_STACK {
            self.matches_from_stack(seed, max_cost, &mut f);
        } else {
            self.matches_from(seed, max_cost, 0, 0, 0, &mut f);
        }
    }
}
