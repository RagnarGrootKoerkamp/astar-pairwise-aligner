// Type for values of contours
type Layer = usize;

pub mod map {
    use super::*;
    use std::collections::BTreeMap;
    use std::ops::Bound::{Excluded, Included, Unbounded};
    use std::ops::RangeFull;
    pub struct Thresholds<K, V> {
        pub m: BTreeMap<K, (Layer, V)>,
    }

    // K: y-coordinate
    // V: Layer metadata
    impl<K, V> Default for Thresholds<K, V> {
        fn default() -> Self {
            Self {
                m: Default::default(),
            }
        }
    }

    impl<K: Ord + Copy + std::fmt::Debug, V: Copy + std::fmt::Debug> Thresholds<K, V> {
        /// Set f(x) = y.
        /// Only inserts if y is larger than the current value at x.
        /// Returns whether insertion took place.
        #[inline]
        pub fn set(&mut self, x: K, y: (Layer, V)) -> bool {
            //println!("Set {:?} to {:?}", x, y);
            let cur_val = self.get(x);
            if cur_val.map_or(false, |c| y.0 <= c.0) {
                //println!("Set {:?} to {:?} -> SKIP", x, y);
                return false;
            }
            // Delete elements right of x at most y.
            let to_remove = self
                .m
                .range((Excluded(x), Unbounded))
                .take_while(|&(_, &value)| value.0 <= y.0)
                .map(|(&key, _)| key)
                .collect::<Vec<_>>();
            for key in to_remove {
                self.m.remove(&key);
            }
            self.m.insert(x, y);
            true
        }

        /// Get the largest value in the map.
        #[inline]
        pub fn max(&self) -> Option<(Layer, V)> {
            self.m.range(RangeFull).next_back().map(|(_, y)| *y)
        }

        /// Get f(x): the y for the largest key <= x inserted into the map.
        #[inline]
        pub fn get(&self, x: K) -> Option<(Layer, V)> {
            let v = self
                .m
                .range((Unbounded, Included(x)))
                .next_back()
                .map(|(_, y)| *y);
            //println!("Get {:?} = {:?}", x, v);
            v
        }

        /// f(x') for the smallest x' > x inserted into the map.
        #[inline]
        pub fn get_larger(&self, x: K) -> Option<(Layer, V)> {
            self.m
                .range((Excluded(x), Unbounded))
                .next()
                .map(|(_, y)| *y)
        }
    }
}

pub mod vec {
    use super::*;
    use std::cmp::{Ord, Ordering};

    // Based on:
    // A fast algorithm for computing LCS, 1977, Hunt & Szymanski
    pub struct Thresholds<K, V> {
        // t[layer] contains the position of the contour at that layer.
        // K: y-coordinate of contour
        // V: Metadata to store for the dominant points.
        pub t: Vec<(K, V)>,
    }

    // K: y-coordinate
    // V: Layer metadata
    impl<K, V> Default for Thresholds<K, V> {
        fn default() -> Self {
            Self {
                t: Default::default(),
            }
        }
    }

    impl<K: Ord + Copy + std::fmt::Debug, V: Copy + std::fmt::Debug> Thresholds<K, V> {
        /// Set f(x) = y.
        /// Only inserts if y is larger than the current value at x.
        /// Returns whether insertion took place.
        #[inline]
        pub fn set(&mut self, y: K, (layer, v): (Layer, V)) -> bool {
            assert!(layer <= self.t.len());
            if layer > 0 {
                assert!(self.t[layer - 1].0 <= y);
            }

            if layer == self.t.len() {
                self.t.push((y, v));
                return true;
            }

            //println!("Set {:?} to {:?}", x, y);
            let cur = &mut self.t[layer];
            assert!(y < cur.0);
            *cur = (y, v);
            true
        }

        /// Get the largest value in the map.
        #[inline]
        pub fn max(&self) -> Option<(Layer, V)> {
            self.t.last().map(|(_, y)| (self.t.len() - 1, *y))
        }

        /// f(y') for the largest y' <= y inserted into the map.
        #[inline]
        pub fn get(&self, y: K) -> Option<(Layer, V)> {
            let layer = self
                .t
                .binary_search_by(|&(k, _)| {
                    if k <= y {
                        Ordering::Less
                    } else {
                        Ordering::Greater
                    }
                })
                .unwrap_err();
            if layer == 0 {
                None
            } else {
                Some((layer - 1, self.t[layer - 1].1))
            }
        }

        /// f(y') for the smallest y' > y inserted into the map.
        #[inline]
        pub fn get_larger(&self, y: K) -> Option<(Layer, V)> {
            let layer = self
                .t
                .binary_search_by(|&(k, _)| {
                    if k <= y {
                        Ordering::Less
                    } else {
                        Ordering::Greater
                    }
                })
                .unwrap_err();
            Some((layer, self.t.get(layer)?.1))
        }
    }
}
