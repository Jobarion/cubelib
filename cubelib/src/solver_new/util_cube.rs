use std::cmp::Ordering;
use crate::cube::{Cube333, CubeAxis, Transformation333};
use crate::cube::turn::TransformableMut;
use crate::steps::coord::Coord;
use crate::steps::dr::coords::DRUDEOFBCoord;
use crate::steps::eo::coords::BadEdgeCount;
use crate::steps::finish::coords::HTRFinishCoord;
use crate::steps::fr::coords::FRUDWithSliceCoord;
use crate::steps::htr::coords::HTRDRUDCoord;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum CubeState {
    Scrambled,
    EO(Vec<CubeAxis>),
    DR(CubeAxis),
    TripleDR,
    HTR,
    FR(Vec<CubeAxis>),
    Solved
}

impl CubeState {
    fn ordinal(&self) -> u8 {
        match self {
            CubeState::Scrambled => 0,
            CubeState::EO(_) => 1,
            CubeState::DR(_) => 2,
            CubeState::TripleDR => 3,
            CubeState::HTR => 4,
            CubeState::FR(_) => 5,
            CubeState::Solved => 6,
        }
    }
}

fn compare_subset<T: Eq>(a: &Vec<T>, b: &Vec<T>) -> Option<Ordering> {
    if a.len() < b.len() {
        return compare_subset(b, a).map(Ordering::reverse);
    }
    for x in b {
        if !a.contains(x) {
            return None;
        }
    }
    if a.len() == b.len() {
        Some(Ordering::Equal)
    } else {
        Some(Ordering::Greater)
    }
}

impl PartialOrd for CubeState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.ordinal().cmp(&other.ordinal()) {
            Ordering::Less => Some(Ordering::Less),
            Ordering::Equal => {
                match (self, other) {
                    (CubeState::Scrambled, _) => Some(Ordering::Equal),
                    (CubeState::EO(axis0), CubeState::EO(axis1)) => compare_subset(axis0, axis1),
                    (CubeState::DR(axis0), CubeState::DR(axis1)) if axis0 == axis1 => Some(Ordering::Equal),
                    (CubeState::DR(_), CubeState::DR(_)) => None,
                    (CubeState::TripleDR, _) => Some(Ordering::Equal),
                    (CubeState::HTR, _) => Some(Ordering::Equal),
                    (CubeState::FR(axis0), CubeState::FR(axis1)) => compare_subset(axis0, axis1),
                    (CubeState::Solved, _) => Some(Ordering::Equal),
                    _ => unreachable!()
                }
            },
            Ordering::Greater => Some(Ordering::Greater),
        }
    }
}

impl Cube333 {
    pub fn get_cube_state(&self) -> CubeState {
        let mut eo_solved_on = vec![];
        if self.count_bad_edges_fb() == 0 {
            eo_solved_on.push(CubeAxis::FB);
        }
        if self.count_bad_edges_ud() == 0 {
            eo_solved_on.push(CubeAxis::UD);
        }
        if self.count_bad_edges_lr() == 0 {
            eo_solved_on.push(CubeAxis::LR);
        }
        if eo_solved_on.is_empty() {
            return CubeState::Scrambled
        }
        if eo_solved_on.len() < 2 {
            return CubeState::EO(eo_solved_on)
        }
        let dr_axis: Vec<CubeAxis> = [CubeAxis::UD, CubeAxis::FB, CubeAxis::LR].into_iter()
            .zip([Transformation333::Y, Transformation333::X, Transformation333::Z].into_iter())
            .filter_map(|(dr, trans)|{
                let mut cube = self.clone();
                cube.transform(trans);
                if cube.count_bad_edges_fb() == 0 && cube.count_bad_edges_lr() == 0 && DRUDEOFBCoord::from(&cube).val() == 0 {
                    Some(dr)
                } else {
                    None
                }
            })
            .collect();
        assert_ne!(2, dr_axis.len());
        if dr_axis.is_empty() {
            return CubeState::EO(eo_solved_on);
        }
        if dr_axis.len() == 1 || HTRDRUDCoord::from(self).val() != 0 {
            return CubeState::DR(dr_axis[0]);
        }
        if dr_axis.len() == 3 {
            return CubeState::TripleDR;
        }
        if HTRFinishCoord::from(self).val() == 0 {
            return CubeState::Solved;
        }
        let fr_axis: Vec<CubeAxis> = [CubeAxis::UD, CubeAxis::FB, CubeAxis::LR].into_iter()
            .zip([Transformation333::Y, Transformation333::X, Transformation333::Z].into_iter())
            .filter_map(|(fr, trans)|{
                let mut cube = self.clone();
                cube.transform(trans);
                if FRUDWithSliceCoord::from(&cube).val() == 0 {
                    Some(fr)
                } else {
                    None
                }
            })
            .collect();
        CubeState::FR(fr_axis)
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;
    use crate::algs::Algorithm;
    use crate::cube::{Cube333, CubeAxis};
    use crate::solver_new::util_cube::CubeState;

    #[test]
    fn test_dr_no_htr() {
        let cube: Cube333 = Algorithm::from_str("R U2 F2 U2 R").unwrap().into();
        assert_eq!(cube.get_cube_state(), CubeState::DR(vec![CubeAxis::LR]));
    }
}