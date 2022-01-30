use crate::prelude::{Contour, I};

#[derive(Debug, Default, Clone)]
pub struct EqualContour<C1: Contour, C2: Contour> {
    c1: C1,
    c2: C2,
}

impl<C1: Contour, C2: Contour> Contour for EqualContour<C1, C2> {
    fn with_max_len(max_len: I) -> Self {
        Self {
            c1: C1::with_max_len(max_len),
            c2: C2::with_max_len(max_len),
        }
    }

    fn push(&mut self, p: crate::prelude::Pos) {
        self.c1.push(p);
        self.c2.push(p);
    }

    fn contains(&self, q: crate::prelude::Pos) -> bool {
        let a1 = self.c1.contains(q);
        let a2 = self.c2.contains(q);
        assert_eq!(
            a1, a2,
            "Different contains result for {}: {} vs {}\n{:?}\n{:?}\n",
            q, a1, a2, self.c1, self.c2
        );
        a1
    }

    fn is_dominant(&self, q: crate::prelude::Pos) -> bool {
        let a1 = self.c1.is_dominant(q);
        let a2 = self.c2.is_dominant(q);
        assert_eq!(a1, a2);
        a1
    }

    fn prune(&mut self, p: crate::prelude::Pos) -> bool {
        let a = self.c1.prune(p);
        self.c2.prune(p);
        a
    }

    fn prune_filter<F: FnMut(crate::prelude::Pos) -> bool>(&mut self, f: &mut F) -> bool {
        let a = self.c1.prune_filter(f);
        self.c2.prune_filter(f);
        a
    }

    fn len(&self) -> usize {
        self.c1.len()
    }

    fn num_dominant(&self) -> usize {
        let a1 = self.c1.num_dominant();
        let a2 = self.c2.num_dominant();
        assert_eq!(
            a1, a2,
            "Different number of dominant points: {} vs {}\n{:?}\n{:?}\n",
            a1, a2, self.c1, self.c2
        );
        a1
    }
}
