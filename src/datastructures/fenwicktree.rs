#[derive(Debug, Default)]
pub struct FenwickTree {
    data: Vec<usize>,
}

impl FenwickTree {
    pub fn new(n: usize) -> Self {
        FenwickTree {
            data: vec![0; n + 1],
        }
    }

    pub fn new_with_value(n: usize, v: usize) -> Self {
        let mut f = FenwickTree {
            data: vec![0; n + 1],
        };

        let mut b = 1;
        while b <= n {
            let mut i = b;
            while i <= n {
                f.data[i] += v;
                if i + b <= n {
                    f.data[i + b] += f.data[i];
                }
                i += 2 * b;
            }

            b *= 2;
        }
        f
    }

    /// Sum [0, r].
    pub fn query(&self, mut i: usize) -> usize {
        let mut s = 0;
        i += 1;
        while i > 0 {
            s += self.data[i];
            i -= (i as isize & -(i as isize)) as usize;
        }
        s
    }

    /// Add v to position i.
    pub fn add(&mut self, mut i: usize, v: usize) {
        i += 1;
        while i < self.data.len() {
            self.data[i] += v;
            i += (i as isize & -(i as isize)) as usize;
        }
    }

    /// Remove v from position i.
    pub fn remove(&mut self, mut i: usize, v: usize) {
        i += 1;
        while i < self.data.len() {
            self.data[i] -= v;
            i += (i as isize & -(i as isize)) as usize;
        }
    }

    /// Find the first position i s.t. query(i) >= s.
    pub fn search(&self, mut s: usize) -> usize {
        let mut i = 0;
        let mut b = 1 << (usize::BITS - 1 - self.data.len().leading_zeros());
        while b > 0 {
            if i + b < self.data.len() && self.data[i + b] < s {
                s -= self.data[i + b];
                i += b;
            }
            b /= 2;
        }
        i
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fenwicktree() {
        let mut f = FenwickTree::new(8);
        f.add(2, 1);
        assert_eq!(f.query(1), 0);
        assert_eq!(f.query(2), 1);
        assert_eq!(f.query(3), 1);
        assert_eq!(f.search(0), 0);
        assert_eq!(f.search(1), 2);
        assert_eq!(f.search(2), 8);
        f.add(5, 2);
        assert_eq!(f.query(4), 1);
        assert_eq!(f.query(5), 3);
        assert_eq!(f.query(6), 3);
        assert_eq!(f.search(0), 0);
        assert_eq!(f.search(1), 2);
        assert_eq!(f.search(2), 5);
        assert_eq!(f.search(3), 5);
        assert_eq!(f.search(4), 8);
    }

    #[test]
    fn fenwicktree_with_value() {
        let mut f = FenwickTree::new_with_value(8, 1);
        f.remove(2, 1);
        assert_eq!(f.query(0), 1);
        assert_eq!(f.query(1), 2);
        assert_eq!(f.query(2), 2);
        assert_eq!(f.query(3), 3);
        // NOTE: values smaller than the first value all return 0.
        assert_eq!(f.search(0), 0);
        assert_eq!(f.search(1), 0);
        assert_eq!(f.search(2), 1);
        assert_eq!(f.search(3), 3);
    }
}
