use std::hash::Hash;

pub trait Coord<const SIZE: usize>: Into<usize> + Copy + Clone + Eq + PartialEq + Hash {
    fn size() -> usize {
        SIZE
    }
    fn val(&self) -> usize;
}
