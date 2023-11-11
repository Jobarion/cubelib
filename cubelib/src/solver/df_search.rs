use std::cmp::min;

use crate::algs::Algorithm;
use crate::defs::NissSwitchType;
use crate::solver::moveset::{Transition, TransitionTable};
use crate::puzzles::puzzle::{Puzzle, PuzzleMove, Transformable, TransformableMut};
use crate::steps::step::{DefaultStepOptions, StepVariant};

impl DefaultStepOptions {
    pub fn new(min_moves: u8, max_moves: u8, niss_type: NissSwitchType, step_limit: Option<usize>) -> Self {
        DefaultStepOptions {
            min_moves,
            max_moves,
            niss_type,
            step_limit,
        }
    }
}

pub fn dfs_iter<
    'a,
    Turn: PuzzleMove + Transformable<Transformation>,
    Transformation: PuzzleMove,
    PuzzleParam: Puzzle<Turn, Transformation> + 'a,
    TransTable: TransitionTable<Turn>,
    S: StepVariant<Turn, Transformation, PuzzleParam, TransTable> + ?Sized,
>(
    step: &'a S,
    mut cube: PuzzleParam,
    search_opts: DefaultStepOptions,
    mut previous_normal: Option<Turn>,
    mut previous_inverse: Option<Turn>,
    starts_on_normal: bool,
) -> Option<Box<dyn Iterator<Item = Algorithm<Turn>> + 'a>> {
    for t in step.pre_step_trans().iter().cloned() {
        cube.transform(t);
        previous_normal = previous_normal.map(|m|m.transform(t));
        previous_inverse = previous_inverse.map(|m|m.transform(t));
    }

    if !step.is_cube_ready(&cube) {
        return None;
    }

    //Return immediately if the cube is solved. This avoids the issue where we return two solutions if the NISS type is AtStart.
    if step.heuristic(&cube, search_opts.min_moves, search_opts.niss_type != NissSwitchType::Never) == 0 {
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
                let b: Box<dyn Iterator<Item = Algorithm<Turn>>> = match search_opts.niss_type {
                    NissSwitchType::Never if starts_on_normal => {
                        Box::new(
                            next_dfs_level(
                                step,
                                cube.clone(),
                                depth,
                                true,
                                false,
                                previous_normal,
                                previous_inverse,
                            )
                                .map(|alg| alg.reverse()),
                        )
                    },
                    NissSwitchType::Never => {
                        let mut inv_cube = cube.clone();
                        inv_cube.invert();
                        Box::new(
                            next_dfs_level(
                                step,
                                inv_cube,
                                depth,
                                true,
                                false,
                                previous_normal,
                                previous_inverse,
                            )
                                .map(|alg| alg.reverse())
                                .map(|alg| {
                                    Algorithm {
                                        normal_moves: alg.inverse_moves,
                                        inverse_moves: alg.normal_moves
                                    }
                                })
                            ,
                        )
                    },
                    NissSwitchType::Always => Box::new(
                        next_dfs_level(
                            step,
                            cube.clone(),
                            depth,
                            true,
                            true,
                            previous_normal,
                            previous_inverse,
                        )
                            .map(|alg| alg.reverse()),
                    ),
                    NissSwitchType::Before => {
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
            .filter(move |alg| step.is_solution_admissible(&cube, alg))
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
    Turn: PuzzleMove + Transformable<Transformation>,
    Transformation: PuzzleMove,
    PuzzleParam: Puzzle<Turn, Transformation> + 'a,
    TransTable: TransitionTable<Turn>,
    S: StepVariant<Turn, Transformation, PuzzleParam, TransTable> + ?Sized,
>(
    step: &'a S,
    mut cube: PuzzleParam,
    depth_left: u8,
    can_invert: bool,
    invert_allowed: bool,
    previous_normal: Option<Turn>,
    previous_inverse: Option<Turn>,
) -> Box<dyn Iterator<Item = Algorithm<Turn>> + 'a> {
    let lower_bound = if invert_allowed {
        min(1, step.heuristic(&cube, depth_left, invert_allowed))
    } else {
        step.heuristic(&cube, depth_left, invert_allowed)
    };
    let mut inverse = cube.clone();
    let normal_solutions: Box<dyn Iterator<Item = Algorithm<Turn>>> = if depth_left == 0 && lower_bound == 0 {
        Box::new(vec![Algorithm::new()].into_iter())
    } else if lower_bound == 0 || lower_bound > depth_left {
        Box::new(vec![].into_iter())
    } else {
        let state_change_moves = step
            .move_set(&cube, depth_left)
            .st_moves
            .into_iter()
            .cloned()
            .map(move |m| {
                (
                    m,
                    previous_normal
                        .map_or(Transition::any(), |pm| step.move_set(&cube, depth_left).transitions[Into::<usize>::into(pm)].check_move(m))
                )
            })
            .filter(move |(_m, transition_type)| transition_type.allowed && (depth_left != 1 || transition_type.can_end))
            .flat_map(move |(m, t)| {
                cube.turn(m);
                let result = next_dfs_level(
                    step,
                    cube,
                    depth_left - 1,
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
            .move_set(&cube, depth_left)
            .aux_moves
            .into_iter()
            .cloned()
            .map(move |m| {
                (
                    m,
                    previous_normal
                        .map_or(Transition::any(), |pm| step.move_set(&cube, depth_left).transitions[Into::<usize>::into(pm)].check_move(m))
                )
            })
            .filter(move |(_m, transition_type)| transition_type.allowed && (depth_left != 1 || transition_type.can_end))
            .flat_map(move |(m, _)| {
                cube.turn(m);
                let result = next_dfs_level(
                    step,
                    cube,
                    depth_left - 1,
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
    if depth_left > 0 && can_invert && invert_allowed {
        inverse.invert();
        let inverse_solutions = next_dfs_level(
            step,
            inverse,
            depth_left,
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