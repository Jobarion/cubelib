use std::cmp::min;

use crate::algs::Algorithm;
use crate::cube::{Face, FACES, Invertible, Move, Turnable, TURNS};
use crate::cube::Face::*;
use crate::cube::Turn::*;
use crate::moveset::Transition;
use crate::step::StepVariant;

pub const LEGAL_MOVE_COUNT: usize = TURNS.len() * FACES.len();
pub const ALL_MOVES: [Move; LEGAL_MOVE_COUNT] = get_all_moves();

#[derive(Clone, Copy)]
pub struct SearchOptions {
    pub niss_type: NissType,
    pub min_moves: u8,
    pub max_moves: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NissType {
    None,
    AtStart,
    During
}

impl SearchOptions {
    pub fn new(min_moves: u8, max_moves: u8, niss_type: NissType) -> Self {
        SearchOptions {
            min_moves,
            max_moves,
            niss_type
        }
    }
}

// pub fn solve<'a, const SC_SIZE: usize, const AUX_SIZE: usize, const TRANS: usize, C: Turnable + Invertible + Clone + Copy + 'a, H, PC>(
//     step: &'a Step<SC_SIZE, AUX_SIZE, TRANS, C, H, PC>,
//     cube: C,
//     min_moves: u8,
//     max_moves: u8,
//     allow_niss: bool) -> Option<impl Iterator<Item = Algorithm> + 'a>
//     where
//         H: Fn(&C) -> u8,
//         PC: Fn(&C) -> bool {
//
//     if (step.pre_check)(&cube) {
//         //Transform cube
//         Some(dfs_iter(&step.move_set, step.heuristic.clone(), cube, min_moves, max_moves, allow_niss))
//         //Transform solutions
//     } else {
//         None
//     }
// }

pub fn dfs_iter<
    'a,
    const SC_SIZE: usize,
    const AUX_SIZE: usize,
    C: Turnable + Invertible + Clone + Copy + 'a,
    S: StepVariant<SC_SIZE, AUX_SIZE, C> + ?Sized
>(
    step: &'a S,
    mut cube: C,
    search_opts: SearchOptions) -> Option<impl Iterator<Item = Algorithm> + 'a> {

    for t in step.pre_step_trans().iter().cloned() {
        cube.transform(t);
    }

    if !step.is_cube_ready(&cube) {
        return None;
    }

    Some((search_opts.min_moves..=search_opts.max_moves).into_iter()
        .flat_map(move |depth| {
            let b: Box::<dyn Iterator<Item = Algorithm>> = match search_opts.niss_type {

                NissType::During | NissType::None => {
                    Box::new(next_dfs_level(step, cube.clone(), depth, true, search_opts.niss_type == NissType::During, None)
                        .map(|alg| alg.reverse()))
                },

                NissType::AtStart => {
                    let no_niss = next_dfs_level(step, cube.clone(), depth, true, false, None)
                        .map(|alg| alg.reverse());
                    let mut inverted = cube.clone();
                    inverted.invert();
                    let only_niss = next_dfs_level(step, inverted, depth, true, false, None)
                        .map(|alg| alg.reverse())
                        .map(|alg| Algorithm { normal_moves: alg.inverse_moves, inverse_moves: alg.normal_moves });
                    Box::new(no_niss.chain(only_niss))
                }
            };
            b
        })
        .map(|mut alg| {
            for t in step.pre_step_trans().iter().cloned().rev() {
                alg.transform(t);
            }
            alg
        }))
}

fn next_dfs_level<
    'a,
    const SC_SIZE: usize,
    const AUX_SIZE: usize,
    C: Turnable + Invertible + Copy + Clone + 'a,
    S: StepVariant<SC_SIZE, AUX_SIZE, C> + ?Sized>(
    step: &'a S,
    mut cube: C,
    depth: u8,
    can_invert: bool,
    invert_allowed: bool,
    previous: Option<Move>) -> Box<dyn Iterator<Item = Algorithm> + 'a>  {

    let lower_bound = if invert_allowed {
        min(1, step.heuristic(&cube))
    } else {
        step.heuristic(&cube)
    };

    // println!("{depth} {invert_allowed} {can_invert} {lower_bound}");

    let mut inverse = cube.clone();
    let normal_solutions: Box::<dyn Iterator<Item = Algorithm>> = if depth == 0 && lower_bound == 0 {
        // println!("\tSolved");
        Box::new(vec![Algorithm::new()].into_iter())
    } else if lower_bound == 0 || lower_bound > depth {
        // println!("\tSkipped");
        Box::new(vec![].into_iter())
    } else {
        let state_change_moves = step.move_set().st_moves.into_iter()
            // .filter(move |m| (depth == 5 && m.to_id() == Move::L.to_id()) ||
            //         (depth == 4 && m.to_id() == Move::B.to_id()) ||
            //         (depth == 3 && m.to_id() == Move::Li.to_id()) ||
            //         (depth == 2 && m.to_id() == Move::U.to_id()) ||
            //         (depth == 1 && m.to_id() == Move::B.to_id()))
            .map(move |m| (m, previous.map(|pm| {
                // if depth == 3 {
                //     println!("{pm} {m}");
                // }
                step.move_set().transitions[Into::<usize>::into(&pm)].check_move(&m)
            }).unwrap_or(Transition::any())))
            .filter(move |(m, transition_type)| {
                // if depth == 4 {
                //     println!("Depth {depth} move {m} allowed {} can end {}", transition_type.allowed, transition_type.can_end);
                // }
                transition_type.allowed && (depth != 1 || transition_type.can_end)
            })
            // .filter(move |(m, _)| {
            //         (depth == 3 && invert_allowed && m.to_id() == Move::U.to_id()) ||
            //         (depth == 2 && invert_allowed && m.to_id() == Move::D.to_id()) ||
            //         (depth == 1 && !invert_allowed && m.to_id() == Move::D.to_id())
            // })
            .flat_map(move |(m, t)|{
                cube.turn(m);
                // println!("{depth} applying sc {m} {}", t.can_end);
                let result = next_dfs_level(step, cube, depth - 1, t.can_end, invert_allowed, Some(m));
                cube.turn(m.invert());
                result.map(move |mut alg|{
                    alg.normal_moves.push(m);
                    alg
                })
            });
        let aux_moves = step.move_set().aux_moves.into_iter()
            // .filter(move |m| (depth == 5 && m.to_id() == Move::L.to_id()) ||
            //     (depth == 4 && m.to_id() == Move::B.to_id()) ||
            //     (depth == 3 && m.to_id() == Move::Li.to_id()) ||
            //     (depth == 2 && m.to_id() == Move::U.to_id()) ||
            //     (depth == 1 && m.to_id() == Move::B.to_id()))
            .map(move |m| (m, previous.map(|pm| {
                // if depth == 3 {
                //     println!("{pm} {m}");
                // }
                step.move_set().transitions[Into::<usize>::into(&pm)].check_move(&m)
            }).unwrap_or(Transition::any())))
            .filter(move |(m, transition_type)| {
                // if depth == 3 {
                //     println!("Depth {depth} move {m} allowed {} can end {}", transition_type.allowed, transition_type.can_end);
                // }
                transition_type.allowed && (depth != 1 || transition_type.can_end)
            })
            // .filter(move |(m, _)| depth == 4 && invert_allowed && m.to_id() == Move::F2.to_id())
            .flat_map(move |(m, _)|{
                cube.turn(m);
                // println!("{depth} applying aux {m}");
                // let next_skip = skip_move_set_normal.apply_move(m.0);
                let result = next_dfs_level(step, cube, depth - 1, false, invert_allowed, Some(m));
                cube.turn(m.invert());
                result.map(move |mut alg|{
                    alg.normal_moves.push(m);
                    alg
                })
            });
        Box::new(state_change_moves.chain(aux_moves))
    };
    if depth > 0 && can_invert && invert_allowed {
        // println!("{depth} inverting");
        inverse.invert();
        let inverse_solutions = next_dfs_level(step, inverse, depth, false, false, None)
            .map(|alg| Algorithm {
                normal_moves: alg.inverse_moves,
                inverse_moves: alg.normal_moves
            });
        return Box::new(normal_solutions.chain(inverse_solutions));
    } else {
        return normal_solutions;
    };
}

#[derive(Copy, Clone)]
pub struct MoveSkipTracker(u8);

impl MoveSkipTracker {

    const SLICE_MASKS: [u8; 6] = [0b11, 0b11, 0b1100, 0b1100, 0b110000, 0b110000];
    const FACE_MASKS: [u8; 6] = [0b1, 0b11, 0b100, 0b1100, 0b10000, 0b110000];

    pub fn empty() -> MoveSkipTracker {
        MoveSkipTracker(0)
    }

    pub fn is_legal(&self, face: Face) -> bool {
        1 << face as usize & self.0 == 0
    }

    //Never allow U after D, only D after U (and similar for other axis). This prevents solutions with different U D orders and enforces one canonical representation
    pub fn apply_move(&self, face: Face) -> MoveSkipTracker {
        MoveSkipTracker(self.0 & MoveSkipTracker::SLICE_MASKS[face as usize] | MoveSkipTracker::FACE_MASKS[face as usize])
    }
}

const fn get_all_moves() -> [Move; LEGAL_MOVE_COUNT] {
    let mut arr = [Move(Up, Clockwise); 3 * 6];
    let mut f_id = 0;
    while f_id < 6 {
        let mut t_id = 0;
        while t_id < 3 {
            arr[f_id * 3 + t_id] = Move(FACES[f_id], TURNS[t_id]);
            t_id += 1;
        }
        f_id += 1;
    }
    arr
}