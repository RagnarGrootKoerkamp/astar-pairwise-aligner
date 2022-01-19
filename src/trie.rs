use bio::alphabets::{Alphabet, RankTransform};

use crate::prelude::{Cost, I};

pub type State = u32;
pub type Data = u32;

#[derive(Default)]
pub struct TrieNode {
    // Child indices are always positive, so 0 indicates empty.
    children: [State; 4],
    data: Vec<u32>,
    count: u32,
}

pub struct Trie {
    states: Vec<TrieNode>,
    transform: RankTransform,
}

impl Trie {
    pub fn new<'a>(seeds: impl IntoIterator<Item = (&'a [u8], Data)>, alph: &Alphabet) -> Self {
        assert!(
            alph.len() <= 4,
            "Trie can only be used for DNA alphabet for now."
        );
        let mut this = Trie {
            states: Vec::default(),
            transform: RankTransform::new(alph),
        };
        this.states.push(TrieNode::default());
        for (seed, data) in seeds {
            let mut state = 0 as State;
            this.states[state as usize].count += 1;
            for c in seed {
                state = this.force_step(state, *c);
                this.states[state as usize].count += 1;
            }
            this.states[state as usize].data.push(data);
        }
        this
    }

    fn force_step(&mut self, state: State, c: u8) -> State {
        let len = self.states.len();
        match &mut self.states[state as usize].children[self.transform.get(c) as usize] {
            x if *x > 0 => *x,
            x => {
                *x = len as State;
                let x = *x;
                self.states.push(TrieNode::default());
                x
            }
        }
    }

    fn matches_from<F: FnMut(Data, I, Cost)>(
        &self,
        remaining_seed: &[u8],
        max_cost: Cost,
        cost: Cost,
        depth: I,
        state: State,
        f: &mut F,
    ) {
        // No more matches
        if state == 0 {
            return;
        }

        // Accumulated too much cost.
        if cost > max_cost {
            return;
        }

        // Stop here.
        if remaining_seed.is_empty() {
            for d in &self.states[state as usize].data {
                f(*d, depth, cost);
            }
        }

        // Match or substitute a char.
        if let Some((c, remaining_seed)) = remaining_seed.split_first() {
            let matching_index = self.transform.get(*c) as usize;
            for (i, state) in self.states[state as usize].children.iter().enumerate() {
                // TODO: Replace with actual costs.
                let char_cost = if i == matching_index { 0 } else { 1 };
                self.matches_from(
                    remaining_seed,
                    max_cost,
                    cost + char_cost,
                    depth + 1,
                    *state,
                    f,
                );
            }
        }

        // Delete a char: the character in the seed is ignored, and we remain at the same depth.
        if let Some((_c, remaining_seed)) = remaining_seed.split_first() {
            // TODO: Replace with actual costs.
            let deletion_cost = 1;
            self.matches_from(
                remaining_seed,
                max_cost,
                cost + deletion_cost,
                depth,
                state,
                f,
            );
        }

        // Insert a char: No character from the seed is needed, but we increase the depth.
        // NOTE: We never insert the next character in the string, as we could directly match that instead.
        // TODO: Replace with actual costs.
        let deletion_cost = 1;
        let matching_index = remaining_seed
            .first()
            .map(|c| self.transform.get(*c) as usize);
        for (i, state) in self.states[state as usize].children.iter().enumerate() {
            if Some(i) == matching_index {
                continue;
            }
            // TODO: Replace with actual costs.
            self.matches_from(
                remaining_seed,
                max_cost,
                cost + deletion_cost,
                depth + 1,
                *state,
                f,
            );
        }
    }

    /// Finds all matches within the given edit distance, and call `f` for each of them.
    pub fn matches<F: FnMut(Data, I, Cost)>(&self, seed: &[u8], max_cost: Cost, mut f: F) {
        self.matches_from(seed, max_cost, 0, 0, 0, &mut f);
    }
}
