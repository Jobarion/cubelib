use std::cmp::Ordering;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;
use crate::cube::Symmetry;
use crate::cube::turn::ApplySymmetry;

pub trait Coord<const SIZE: usize>: Into<usize> + Copy + Clone + Eq + PartialEq + Hash + Debug {
    fn size() -> usize {
        SIZE
    }
    fn val(&self) -> usize;
    fn wrap(self) -> CoordWrapper<SIZE, Self> {
        CoordWrapper(self)
    }
    fn min_with_symmetries<T: ApplySymmetry + Clone>(t: &T, symmetries: &Vec<Symmetry>) -> Self where for<'a> Self: From<&'a T> {
        symmetries.iter()
            .map(|s|{
                let mut t = t.clone();
                t.apply_symmetry(s);
                Self::from(&t).wrap()
            })
            .min()
            .unwrap()
            .unwrap()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct CoordWrapper<const SIZE: usize, C: Coord<SIZE>>(C);

impl <const SIZE: usize, C: Coord<SIZE>> Deref for CoordWrapper<SIZE, C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl <const SIZE: usize, C: Coord<SIZE>> From<C> for CoordWrapper<SIZE, C> {
    fn from(value: C) -> Self {
        Self(value)
    }
}

impl <const SIZE: usize, C: Coord<SIZE>> CoordWrapper<SIZE, C> {
    pub fn unwrap(self) -> C {
        self.0
    }
}

impl <const SIZE: usize, C: Coord<SIZE>> PartialOrd for CoordWrapper<SIZE, C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl <const SIZE: usize, C: Coord<SIZE>> Ord for CoordWrapper<SIZE, C> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.val().cmp(&other.0.val())
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct ZeroCoord;

impl Into<usize> for ZeroCoord {
    fn into(self) -> usize {
        0
    }
}

impl Coord<0> for ZeroCoord {
    fn val(&self) -> usize {
        0
    }
}

impl <P> From<&P> for ZeroCoord {
    fn from(_: &P) -> Self {
        ZeroCoord
    }
}
