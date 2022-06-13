use itertools::chain;

/// For a given AffineCost<N>, NW and DT use a main M layer, and N affine layers.
/// This struct wraps this and provides an iterator over all layers.
#[derive(Clone)]
pub struct Layers<const N: usize, T> {
    pub m: T,
    pub affine: [T; N],
}

impl<const N: usize, T> Layers<N, T> {
    pub fn new(m: T) -> Self
    where
        T: Clone,
    {
        let affine = [(); N].map(|_| m.clone());
        Self { m, affine }
    }
}

impl<'a, const N: usize, T> IntoIterator for &'a Layers<N, T> {
    type Item = &'a T;

    type IntoIter = std::iter::Chain<std::array::IntoIter<&'a T, 1_usize>, std::slice::Iter<'a, T>>;

    fn into_iter(self) -> Self::IntoIter {
        chain([&self.m], self.affine.iter()).into_iter()
    }
}

impl<'a, const N: usize, T> IntoIterator for &'a mut Layers<N, T> {
    type Item = &'a mut T;

    type IntoIter =
        std::iter::Chain<std::array::IntoIter<&'a mut T, 1_usize>, std::slice::IterMut<'a, T>>;

    fn into_iter(self) -> Self::IntoIter {
        chain([&mut self.m], self.affine.iter_mut()).into_iter()
    }
}
