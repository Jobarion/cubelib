

use log::{debug, error, warn};
use std::collections::{HashMap};
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::marker::PhantomData;

use crate::coord::Coord;
use crate::cube::{NewSolved, Turnable};
use crate::moveset::{MoveSet, TransitionTable};

pub struct PruningTable<const C_SIZE: usize, C: Coord<C_SIZE>> {
    entries: Box<[u8; C_SIZE]>,
    coord_type: PhantomData<C>,
}

impl<const C_SIZE: usize, C: Coord<C_SIZE>> PruningTable<C_SIZE, C> {
    pub fn new() -> Self {
        PruningTable {
            entries: vec![0xFF; C_SIZE].into_boxed_slice().try_into().unwrap(),
            coord_type: PhantomData,
        }
    }

    pub fn get(&self, id: C) -> Option<u8> {
        let id: usize = id.into();
        let entry = self.entries[id];
        match entry {
            0xFF => None,
            n => Some(n),
        }
    }

    pub fn set(&mut self, id: C, entry: u8) {
        let id: usize = id.into();
        self.entries[id] = entry;
    }
}

pub fn generate<
    const COORD_SIZE: usize,
    Mapper,
    CubeParam: Turnable + NewSolved + Clone,
    CoordParam: Coord<COORD_SIZE> + Copy + Hash + Eq + Debug,
    const ST_MOVES: usize,
    const AUX_MOVES: usize,
>(
    move_set: &MoveSet<ST_MOVES, AUX_MOVES>,
    mapper: &Mapper,
) -> PruningTable<COORD_SIZE, CoordParam>
where
    Mapper: Fn(&CubeParam) -> CoordParam,
{
    let aux_moveset = MoveSet {
        aux_moves: move_set.aux_moves,
        st_moves: [],
        transitions: [TransitionTable::new(0, 0); 18],
    };
    let start = CubeParam::new_solved();
    let mut visited = HashMap::new();
    let mut to_check = vec![start.clone()];
    visited.insert(mapper(&start), start);
    while !to_check.is_empty() {
        to_check = pre_gen_coset_0(&aux_moveset, mapper, &mut visited, &to_check);
    }

    let mut to_check = HashMap::new();
    let mut table = PruningTable::new();
    for (start_coord, start_cube) in visited {
        table.set(start_coord, 0);
        to_check.insert(start_coord, start_cube);
    }
    debug!("Found {} variations of the goal state", to_check.len());
    let mut total_checked = 0;
    for depth in 0..20 {
        total_checked += to_check.len();
        debug!(
            "Checked {:width$}/{} cubes at depth {depth}",
            total_checked,
            CoordParam::size(),
            width = CoordParam::size().to_string().len()
        );
        to_check = fill_table(&move_set, &mut table, depth, mapper, to_check);
        if to_check.is_empty() {
            break;
        }
    }
    total_checked += to_check.len();
    if total_checked != CoordParam::size() {
        warn!(
            "Expected {} cubes in table but got {total_checked}. The coordinate may be malformed",
            CoordParam::size()
        );
    }
    table
}

fn pre_gen_coset_0<
    const COORD_SIZE: usize,
    Mapper,
    CubeParam: Turnable + NewSolved + Clone,
    CoordParam: Coord<COORD_SIZE> + Copy + Hash + Eq + Debug,
    const AUX_MOVES: usize,
>(
    move_set: &MoveSet<0, AUX_MOVES>,
    mapper: &Mapper,
    visited: &mut HashMap<CoordParam, CubeParam>,
    to_check: &Vec<CubeParam>,
) -> Vec<CubeParam>
where
    Mapper: Fn(&CubeParam) -> CoordParam,
{
    let mut check_next = vec![];
    for cube in to_check {
        for m in move_set.aux_moves {
            let mut cube = cube.clone();
            cube.turn(m);
            let coord = mapper(&cube);
            if visited.contains_key(&coord) {
                continue;
            }
            visited.insert(coord, cube.clone());
            check_next.push(cube);
        }
    }
    check_next
}

pub fn generate_pure<
    const COORD_SIZE: usize,
    Mapper,
    CubeParam: Turnable + NewSolved + Clone,
    CoordParam: Coord<COORD_SIZE> + Copy + Hash + Eq + Debug,
    const ST_MOVES: usize,
    const AUX_MOVES: usize,
>(
    move_set: &MoveSet<ST_MOVES, AUX_MOVES>,
    mapper: &Mapper,
) -> PruningTable<COORD_SIZE, CoordParam>
where
    Mapper: Fn(&CubeParam) -> CoordParam,
{
    let mut table = PruningTable::new();
    let mut to_check: HashMap<CoordParam, CubeParam> = HashMap::new();
    let start_cube = CubeParam::new_solved();
    let start_coord = mapper(&start_cube);
    table.set(start_coord, 0);
    to_check.insert(start_coord, start_cube);
    let mut total_checked = 0;
    for depth in 0..20 {
        total_checked += to_check.len();
        debug!(
            "Checked {:width$}/{} cubes at depth {depth}",
            total_checked,
            CoordParam::size(),
            width = CoordParam::size().to_string().len()
        );
        to_check = fill_table(move_set, &mut table, depth, &mapper, to_check);
        if to_check.is_empty() {
            break;
        }
    }
    if total_checked < CoordParam::size() - 1 {
        warn!(
            "Expected {} cubes in table but got {total_checked}. The coordinate may be malformed",
            CoordParam::size()
        );
    }
    table
}

fn fill_table<
    const COORD_SIZE: usize,
    Mapper,
    CubeParam: Turnable + NewSolved + Clone,
    CoordParam: Coord<COORD_SIZE> + Copy + Hash + Eq + Debug,
    const ST_MOVES: usize,
    const AUX_MOVES: usize,
>(
    move_set: &MoveSet<ST_MOVES, AUX_MOVES>,
    table: &mut PruningTable<COORD_SIZE, CoordParam>,
    depth: u8,
    mapper: &Mapper,
    to_check: HashMap<CoordParam, CubeParam>,
) -> HashMap<CoordParam, CubeParam>
where
    Mapper: Fn(&CubeParam) -> CoordParam,
{
    let mut next_cubes: HashMap<CoordParam, CubeParam> = HashMap::new();
    for (_coord, cube) in to_check.into_iter() {
        for m in move_set
            .aux_moves
            .into_iter()
            .chain(move_set.st_moves.into_iter())
        {
            let mut cube = cube.clone();
            cube.turn(m);
            let coord = mapper(&cube);
            let stored = table.get(coord);
            if stored == None {
                table.set(coord, depth + 1);
                next_cubes.insert(coord, cube);
            }
        }
    }
    next_cubes
}

pub fn generate_debug<
    const COORD_SIZE: usize,
    Mapper,
    CubeParam: Turnable + NewSolved + Clone + Debug + Display,
    CoordParam: Coord<COORD_SIZE> + Copy + Hash + Eq + Debug,
    const ST_MOVES: usize,
    const AUX_MOVES: usize,
>(
    move_set: &MoveSet<ST_MOVES, AUX_MOVES>,
    mapper: &Mapper,
) -> PruningTable<COORD_SIZE, CoordParam>
where
    Mapper: Fn(&CubeParam) -> CoordParam,
{
    let mut table = PruningTable::new();
    let mut to_check: HashMap<CoordParam, Vec<CubeParam>> = HashMap::new();
    let start_cube = CubeParam::new_solved();
    let start_coord = mapper(&start_cube);
    table.set(start_coord, 0);
    to_check.insert(start_coord, vec![start_cube]);
    let mut total_checked = 0;
    for depth in 0..20 {
        total_checked += to_check.len();
        debug!(
            "Checked {}/{} cubes at depth {depth}",
            total_checked,
            CoordParam::size()
        );
        to_check = fill_table_debug(move_set, &mut table, depth, &mapper, to_check);
        if to_check.is_empty() {
            break;
        }
    }
    if total_checked < CoordParam::size() - 1 {
        warn!(
            "Expected {} cubes in table but got {total_checked}. The coordinate may be malformed",
            CoordParam::size() - 1
        )
    }
    table
}

fn fill_table_debug<
    const COORD_SIZE: usize,
    Mapper,
    CubeParam: Turnable + NewSolved + Clone + Debug + Display,
    CoordParam: Coord<COORD_SIZE> + Copy + Hash + Eq + Debug,
    const ST_MOVES: usize,
    const AUX_MOVES: usize,
>(
    move_set: &MoveSet<ST_MOVES, AUX_MOVES>,
    table: &mut PruningTable<COORD_SIZE, CoordParam>,
    depth: u8,
    mapper: &Mapper,
    to_check: HashMap<CoordParam, Vec<CubeParam>>,
) -> HashMap<CoordParam, Vec<CubeParam>>
where
    Mapper: Fn(&CubeParam) -> CoordParam,
{
    let mut next_cubes: HashMap<CoordParam, Vec<CubeParam>> = HashMap::new();
    for (coord_pre, mut cubes) in to_check.into_iter() {
        for m in move_set
            .aux_moves
            .into_iter()
            .chain(move_set.st_moves.into_iter())
        {
            let mut cube = cubes[0].clone();
            let pre = cube.clone();
            cube.turn(m);
            let coord = mapper(&cube);
            for other in cubes.iter_mut() {
                let pre_other = other.clone();
                other.turn(m);
                let other_coord = mapper(&other);
                if other_coord != coord {
                    error!(
                        "Coord diff. Pre {:?} A: {:?} B: {:?} move {m}",
                        coord_pre, coord, other_coord
                    );
                    error!(
                        "Pre A\n{}\nA\n{}\nPre B\n{}\nB\n{}",
                        pre, cube, pre_other, other
                    );
                    panic!()
                }
            }
            let stored = table.get(coord);
            if stored == None {
                table.set(coord, depth + 1);
                next_cubes.insert(coord, cubes.clone());
            } else if next_cubes.contains_key(&coord) {
                next_cubes.get_mut(&coord).unwrap().push(cube);
            }
        }
    }
    next_cubes
}
