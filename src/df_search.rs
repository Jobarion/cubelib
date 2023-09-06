use std::cell::{Ref, RefCell};
use std::cmp::min;
use std::rc::Rc;
use std::str::FromStr;
use rand::Rng;
use crate::algs::Algorithm;
use crate::cube::{Cube, Face, FACES, Invertible, Move, Turn, Turnable, TURNS};
use crate::cube::Face::*;
use crate::cube::Turn::*;
use crate::cubie::CubieCube;
use crate::moveset::{MoveSet, Transition};

pub const LEGAL_MOVE_COUNT: usize = TURNS.len() * FACES.len();
pub const ALL_MOVES: [Move; LEGAL_MOVE_COUNT] = get_all_moves();

pub fn dfs_iter<'a, const SC_SIZE: usize, const AUX_SIZE: usize, C: Turnable + Invertible + Clone + Copy + 'a, H>(
    move_set: &'a MoveSet<SC_SIZE, AUX_SIZE>,
    heuristic: Rc<H>,
    cube: C,
    allow_niss: bool) -> impl Iterator<Item = Algorithm> + 'a where H: Fn(&C) -> u32 + 'a {
    (0..20).into_iter()
        .flat_map(move |depth| {
            println!("Analyzing depth {}", depth);
            next_dfs_level(move_set, heuristic.clone(), cube.clone(), depth, true, allow_niss, None)
                .map(|alg| alg.reverse())
        })
}

fn next_dfs_level<'a, const SC_SIZE: usize, const AUX_SIZE: usize, C: Turnable + Invertible + Copy + Clone + 'a, H>(
    move_set: &'a MoveSet<SC_SIZE, AUX_SIZE>,
    heuristic: Rc<H>,
    mut cube: C,
    depth: u32,
    can_invert: bool,
    invert_allowed: bool,
    previous: Option<Move>) -> Box<dyn Iterator<Item = Algorithm> + 'a> where H: Fn(&C) -> u32 + 'a {

    let lower_bound = if invert_allowed {
        min(1, heuristic(&cube))
    } else {
        heuristic(&cube)
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
        let h_sc = heuristic.clone();
        let state_change_moves = move_set.st_moves.into_iter()
            .map(move |m| (m, previous.map(|p| move_set.transitions[Into::<usize>::into(&p)].check_move(&m)).unwrap_or(Transition::any())))
            .filter(move |(_, transition_type)| transition_type.allowed && (depth != 1 || transition_type.can_end))
            // .filter(move |(m, _)| {
            //         (depth == 3 && invert_allowed && m.to_id() == Move::U.to_id()) ||
            //         (depth == 2 && invert_allowed && m.to_id() == Move::D.to_id()) ||
            //         (depth == 1 && !invert_allowed && m.to_id() == Move::D.to_id())
            // })
            .flat_map(move |(m, t)|{
                cube.turn(m);
                // println!("{depth} applying sc {m} {}", t.can_end);
                let result = next_dfs_level(move_set, h_sc.clone(), cube, depth - 1, t.can_end, invert_allowed, Some(m));
                cube.turn(m.invert());
                result.map(move |mut alg|{
                    alg.normal_moves.push(m);
                    alg
                })
            });
        let h_aux = heuristic.clone();
        let aux_moves = move_set.aux_moves.into_iter()
            .map(move |m| (m, previous.map(|p| move_set.transitions[Into::<usize>::into(&p)].check_move(&m)).unwrap_or(Transition::any())))
            .filter(move |(_, transition_type)| transition_type.allowed && (depth != 1 || transition_type.can_end))
            // .filter(move |(m, _)| depth == 4 && invert_allowed && m.to_id() == Move::F2.to_id())
            .flat_map(move |(m, _)|{
                cube.turn(m);
                // println!("{depth} applying aux {m}");
                // let next_skip = skip_move_set_normal.apply_move(m.0);
                let result = next_dfs_level(move_set, h_aux.clone(), cube, depth - 1, false, invert_allowed, Some(m));
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
        let inverse_solutions = next_dfs_level(move_set, heuristic.clone(), inverse, depth, false, false, None)
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