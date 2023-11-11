use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use log::{debug, warn};

use crate::solver::moveset::{MoveSet, TransitionTable};
use crate::puzzles::puzzle::{Puzzle, PuzzleMove, Transformable};
use crate::solver::lookup_table::TableSource::Loaded;
use crate::steps::coord::Coord;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum TableSource {
    Generated,
    Loaded
}

pub struct PruningTable<const C_SIZE: usize, C: Coord<C_SIZE>> {
    entries: Box<[u8; C_SIZE]>,
    coord_type: PhantomData<C>,
    source: TableSource,
}

impl<const C_SIZE: usize, C: Coord<C_SIZE>> PruningTable<C_SIZE, C> {
    pub fn new() -> Self {
        PruningTable {
            entries: vec![0xFF; C_SIZE].into_boxed_slice().try_into().unwrap(),
            coord_type: PhantomData,
            source: TableSource::Generated
        }
    }

    pub fn load(data: Vec<u8>) -> Self {
        assert_eq!(data.len(), C_SIZE);
        PruningTable {
            entries: data.into_boxed_slice().try_into().unwrap(),
            coord_type: PhantomData,
            source: Loaded
        }
    }

    pub fn get_source(&self) -> TableSource {
        self.source
    }

    pub fn get_bytes(&self) -> &[u8; C_SIZE] {
        &*self.entries
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
    Turn: PuzzleMove + Transformable<Transformation>,
    Transformation: PuzzleMove,
    PuzzleParam: Puzzle<Turn, Transformation>,
    TransTable: TransitionTable<Turn>,
    CoordParam: Coord<COORD_SIZE> + Copy + Hash + Eq + Debug,
>(
    move_set: &MoveSet<Turn, TransTable>,
    mapper: &Mapper,
) -> PruningTable<COORD_SIZE, CoordParam>
where
    Mapper: Fn(&PuzzleParam) -> CoordParam,
{
    let start = PuzzleParam::default();
    let mut visited = HashMap::new();
    let mut to_check = vec![start.clone()];
    visited.insert(mapper(&start), start);
    while !to_check.is_empty() {
        to_check = pre_gen_coset_0(&move_set, mapper, &mut visited, &to_check);
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
    Turn: PuzzleMove + Transformable<Transformation>,
    Transformation: PuzzleMove,
    PuzzleParam: Puzzle<Turn, Transformation>,
    TransTable: TransitionTable<Turn>,
    CoordParam: Coord<COORD_SIZE> + Copy + Hash + Eq + Debug,
>(
    move_set: &MoveSet<Turn, TransTable>,
    mapper: &Mapper,
    visited: &mut HashMap<CoordParam, PuzzleParam>,
    to_check: &Vec<PuzzleParam>,
) -> Vec<PuzzleParam>
where
    Mapper: Fn(&PuzzleParam) -> CoordParam,
{
    let mut check_next = vec![];
    for cube in to_check {
        for m in move_set.aux_moves.iter().cloned() {
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
    Turn: PuzzleMove + Transformable<Transformation>,
    Transformation: PuzzleMove,
    PuzzleParam: Puzzle<Turn, Transformation>,
    TransTable: TransitionTable<Turn>,
    CoordParam: Coord<COORD_SIZE> + Copy + Hash + Eq + Debug,
    const ST_MOVES: usize,
    const AUX_MOVES: usize,
>(
    move_set: &MoveSet<Turn, TransTable>,
    mapper: &Mapper,
) -> PruningTable<COORD_SIZE, CoordParam>
where
    Mapper: Fn(&PuzzleParam) -> CoordParam,
{
    let mut table = PruningTable::new();
    let mut to_check: HashMap<CoordParam, PuzzleParam> = HashMap::new();
    let start_cube = PuzzleParam::default();
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
    Turn: PuzzleMove + Transformable<Transformation>,
    Transformation: PuzzleMove,
    PuzzleParam: Puzzle<Turn, Transformation>,
    TransTable: TransitionTable<Turn>,
    CoordParam: Coord<COORD_SIZE> + Copy + Hash + Eq + Debug,
>(
    move_set: &MoveSet<Turn, TransTable>,
    table: &mut PruningTable<COORD_SIZE, CoordParam>,
    depth: u8,
    mapper: &Mapper,
    to_check: HashMap<CoordParam, PuzzleParam>,
) -> HashMap<CoordParam, PuzzleParam>
where
    Mapper: Fn(&PuzzleParam) -> CoordParam,
{
    let mut next_cubes: HashMap<CoordParam, PuzzleParam> = HashMap::new();
    for (_coord, cube) in to_check.into_iter() {
        for m in move_set
            .aux_moves
            .into_iter()
            .chain(move_set.st_moves.into_iter())
            .cloned()
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
