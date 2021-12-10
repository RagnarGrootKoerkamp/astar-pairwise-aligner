use crate::util::*;

pub struct SeedMatches {
    l: usize,
    len_a: usize,
    seed_qgrams: Vec<(usize, usize)>,
    qgram_index: QGramIndex,
}

impl SeedMatches {
    pub fn iter<'a>(&'a self) -> impl DoubleEndedIterator<Item = Pos> + 'a {
        self.seed_qgrams
            .iter()
            .map(move |&(i, seed)| {
                let matches = self.qgram_index.qgram_matches(seed);
                (i, matches)
            })
            .map(|(x, ys)| ys.iter().map(move |&y| Pos(x, y)))
            .flatten()
    }

    pub fn potential(&self, Pos(i, _): Pos) -> usize {
        self.len_a / self.l - min(i + self.l - 1, self.len_a) / self.l
    }

    pub fn num_matches(&self) -> usize {
        self.seed_qgrams
            .iter()
            .map(|&(i, seed)| self.qgram_index.qgram_matches(seed).len())
            .sum()
    }
}

pub fn find_matches<'a>(
    a_text: &'a Sequence,
    b_text: &'a Sequence,
    text_alphabet: &Alphabet,
    l: usize,
) -> SeedMatches {
    // Convert to a binary sequences.
    let rank_transform = RankTransform::new(text_alphabet);

    // Split a into seeds of size l, which are encoded as `usize`.
    let seed_qgrams: Vec<(usize, usize)> = a_text
        .chunks_exact(l)
        .enumerate()
        .map(|(i, s)| (l * i, s))
        // .intersperse_with({
        //     let mut iter = a_text[l / 2..]
        //         .chunks_exact(l)
        //         .enumerate()
        //         .map(|(i, s)| (l * i + l / 2, s));
        //     move || iter.next().unwrap()
        // })
        // A chunk of size l has exactly one qgram of length l.
        .map(|(i, seed)| (i, rank_transform.qgrams(l as u32, seed).next().unwrap()))
        .collect::<Vec<_>>();

    // Find matches of the seeds of a in b.
    // NOTE: This uses O(alphabet^l) memory.
    let qgram_index = QGramIndex::new(l as u32, b_text, &text_alphabet);

    SeedMatches {
        l,
        len_a: a_text.len(),
        seed_qgrams,
        qgram_index,
    }
}
