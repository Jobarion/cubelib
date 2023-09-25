
use std::cmp::min;

use crate::algs::Algorithm;
use crate::cube::Face::*;
use crate::cube::Turn::*;
use crate::cube::{Face, Invertible, Move, Turnable, FACES, TURNS};
use crate::moveset::Transition;
use crate::steps::step::StepVariant;

pub const LEGAL_MOVE_COUNT: usize = TURNS.len() * FACES.len();
pub const ALL_MOVES: [Move; LEGAL_MOVE_COUNT] = get_all_moves();

#[derive(Clone, Copy)]
pub struct SearchOptions {
    pub niss_type: NissType,
    pub min_moves: u8,
    pub max_moves: u8,
    pub step_limit: Option<usize>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NissType {
    None,
    AtStart,
    During,
}

impl SearchOptions {
    pub fn new(min_moves: u8, max_moves: u8, niss_type: NissType, step_limit: Option<usize>) -> Self {
        SearchOptions {
            min_moves,
            max_moves,
            niss_type,
            step_limit,
        }
    }
}

pub fn dfs_iter<
    'a,
    const SC_SIZE: usize,
    const AUX_SIZE: usize,
    C: Turnable + Invertible + Clone + Copy + 'a,
    S: StepVariant<SC_SIZE, AUX_SIZE, C> + ?Sized,
>(
    step: &'a S,
    mut cube: C,
    search_opts: SearchOptions,
    previous_normal: Option<Move>,
    previous_inverse: Option<Move>,
) -> Option<Box<dyn Iterator<Item = Algorithm> + 'a>> {
    for t in step.pre_step_trans().iter().cloned() {
        cube.transform(t);
    }

    if !step.is_cube_ready(&cube) {
        return None;
    }

    //Return immediately if the cube is solved. This avoids the issue where we return two solutions if the NISS type is AtStart.
    if step.heuristic(&cube) == 0 {
        //Only return a solution if we are allowed to return zero length solutions
        if search_opts.min_moves == 0 {
            return Some(Box::new(vec![Algorithm::new()].into_iter()));
        } else {
            return Some(Box::new(vec![].into_iter()));
        }
    }

    Some(Box::new(
        (search_opts.min_moves..=search_opts.max_moves)
            .into_iter()
            .flat_map(move |depth| {
                let b: Box<dyn Iterator<Item = Algorithm>> = match search_opts.niss_type {
                    NissType::During | NissType::None => Box::new(
                        next_dfs_level(
                            step,
                            cube.clone(),
                            depth,
                            true,
                            search_opts.niss_type == NissType::During,
                            previous_normal,
                            previous_inverse,
                        )
                        .map(|alg| alg.reverse()),
                    ),

                    NissType::AtStart => {
                        let no_niss = next_dfs_level(
                            step,
                            cube.clone(),
                            depth,
                            true,
                            false,
                            previous_normal,
                            previous_inverse,
                        )
                        .map(|alg| alg.reverse());
                        let mut inverted = cube.clone();
                        inverted.invert();
                        let only_niss = next_dfs_level(
                            step,
                            inverted,
                            depth,
                            true,
                            false,
                            previous_inverse,
                            previous_normal,
                        )
                        .map(|alg| alg.reverse())
                        .map(|alg| Algorithm {
                            normal_moves: alg.inverse_moves,
                            inverse_moves: alg.normal_moves,
                        });
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
            }),
    ))
}

fn next_dfs_level<
    'a,
    const SC_SIZE: usize,
    const AUX_SIZE: usize,
    C: Turnable + Invertible + Copy + Clone + 'a,
    S: StepVariant<SC_SIZE, AUX_SIZE, C> + ?Sized,
>(
    step: &'a S,
    mut cube: C,
    depth: u8,
    can_invert: bool,
    invert_allowed: bool,
    previous_normal: Option<Move>,
    previous_inverse: Option<Move>,
) -> Box<dyn Iterator<Item = Algorithm> + 'a> {
    let lower_bound = if invert_allowed {
        min(1, step.heuristic(&cube))
    } else {
        step.heuristic(&cube)
    };

    let mut inverse = cube.clone();
    let normal_solutions: Box<dyn Iterator<Item = Algorithm>> = if depth == 0 && lower_bound == 0 {
        Box::new(vec![Algorithm::new()].into_iter())
    } else if lower_bound == 0 || lower_bound > depth {
        Box::new(vec![].into_iter())
    } else {
        let state_change_moves = step
            .move_set()
            .st_moves
            .into_iter()
            .map(move |m| {
                (
                    m,
                    previous_normal
                        .map(|pm| step.move_set().transitions[Into::<usize>::into(&pm)].check_move(&m))
                        .unwrap_or(Transition::any()),
                )
            })
            .filter(move |(_m, transition_type)| transition_type.allowed && (depth != 1 || transition_type.can_end))
            .flat_map(move |(m, t)| {
                cube.turn(m);
                let result = next_dfs_level(
                    step,
                    cube,
                    depth - 1,
                    t.can_end,
                    invert_allowed,
                    Some(m),
                    previous_inverse,
                );
                cube.turn(m.invert());
                result.map(move |mut alg| {
                    alg.normal_moves.push(m);
                    alg
                })
            });
        let aux_moves = step
            .move_set()
            .aux_moves
            .into_iter()
            .map(move |m| {
                (
                    m,
                    previous_normal
                        .map(|pm| step.move_set().transitions[Into::<usize>::into(&pm)].check_move(&m))
                        .unwrap_or(Transition::any()),
                )
            })
            .filter(move |(_m, transition_type)| transition_type.allowed && (depth != 1 || transition_type.can_end))
            .flat_map(move |(m, _)| {
                cube.turn(m);
                let result = next_dfs_level(
                    step,
                    cube,
                    depth - 1,
                    false,
                    invert_allowed,
                    Some(m),
                    previous_inverse,
                );
                cube.turn(m.invert());
                result.map(move |mut alg| {
                    alg.normal_moves.push(m);
                    alg
                })
            });
        Box::new(state_change_moves.chain(aux_moves))
    };
    if depth > 0 && can_invert && invert_allowed {
        inverse.invert();
        let inverse_solutions = next_dfs_level(
            step,
            inverse,
            depth,
            false,
            false,
            previous_inverse,
            previous_normal,
        )
        .map(|alg| Algorithm {
            normal_moves: alg.inverse_moves,
            inverse_moves: alg.normal_moves,
        });
        return Box::new(normal_solutions.chain(inverse_solutions));
    } else {
        return normal_solutions;
    };
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
