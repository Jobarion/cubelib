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
    DR(Vec<CubeAxis>),
    HTR,
    FR(Vec<CubeAxis>),
    Solved
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
        if dr_axis.is_empty() {
            return CubeState::EO(eo_solved_on);
        }
        if dr_axis.len() < 3 || HTRDRUDCoord::from(self).val() != 0 {
            return CubeState::DR(dr_axis);
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