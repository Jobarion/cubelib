use std::collections::HashMap;
use std::fmt::Debug;
#[cfg(feature = "fs")]
use std::fs;
#[cfg(feature = "fs")]
use std::fs::File;
use std::hash::Hash;
#[cfg(feature = "fs")]
use std::io::{Read, Write};
use std::marker::PhantomData;
#[cfg(feature = "fs")]
use home::home_dir;
use log::{debug, warn};
use num_traits::{FromPrimitive, ToPrimitive};
use crate::solver::moveset::{MoveSet, TransitionTable};
use crate::puzzles::puzzle::{Puzzle, PuzzleMove, Transformable};
use crate::solver::lookup_table::TableSource::Loaded;
use crate::steps::coord::Coord;

const VERSION: u8 = 1;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum TableSource {
    Generated,
    Loaded
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
#[derive(FromPrimitive, ToPrimitive)]
pub enum TableType {
    Uncompressed = 0u8,
    Compressed = 1u8,
    Niss = 2u8
}

#[derive(Clone)]
pub struct PruningTable<const C_SIZE: usize, C: Coord<C_SIZE>> {
    entries: Box<[u8]>,
    coord_type: PhantomData<C>,
    source: TableSource,
    table_type: TableType
}

impl<const C_SIZE: usize, C: Coord<C_SIZE>> PruningTable<C_SIZE, C> {
    pub fn new(table_type: TableType) -> Self {
        let entries = if table_type == TableType::Compressed {
            vec![0xFF; (C_SIZE + 1) / 2].into_boxed_slice().try_into().unwrap()
        } else {
            vec![0xFF; C_SIZE].into_boxed_slice().try_into().unwrap()
        };
        PruningTable {
            entries,
            coord_type: PhantomData,
            source: TableSource::Generated,
            table_type
        }
    }

    pub fn can_niss(&self) -> bool {
        self.table_type == TableType::Niss
    }

    pub fn empty_val(&self) -> u8 {
        match self.table_type {
            TableType::Uncompressed => 0xFF,
            TableType::Compressed | TableType::Niss => 0x0F,
        }
    }


    #[cfg(feature = "fs")]
    pub fn load_from_disk(puzzle_id: &str, table_type: &str) -> Result<Self, String> {
        let mut dir = home_dir().unwrap();
        dir.push(".cubelib");
        dir.push("tables");
        dir.push(puzzle_id);
        dir.push(format!("{table_type}.tbl"));
        debug!("Loading {puzzle_id} {table_type} table from {dir:?}");
        let mut file = File::open(dir).map_err(|e|e.to_string())?;
        let mut buffer = Box::new(Vec::new());
        file.read_to_end(&mut buffer).map_err(|e|e.to_string())?;
        Self::load(buffer)
    }

    #[cfg(feature = "fs")]
    pub fn save_to_disk(&self, puzzle_id: &str, table_type: &str) -> std::io::Result<()> {
        let mut dir = home_dir().unwrap();
        dir.push(".cubelib");
        dir.push("tables");
        dir.push(puzzle_id);
        fs::create_dir_all(dir.clone())?;
        dir.push(format!("{table_type}.tbl"));
        let mut file = File::create(dir)?;
        file.write_all(self.get_bytes().as_slice())?;
        Ok(())
    }

    pub fn load(mut data: Box<Vec<u8>>) -> Result<Self, String> {
        let version = data[0];
        if version != VERSION {
            return Err("Invalid version".to_string())
        }
        let table_type: TableType = TableType::from_u8(data[1]).unwrap();
        debug!("Table type {table_type:?}");
        data.drain(0..2);
        if table_type == TableType::Compressed {
            assert_eq!(data.len(), (C_SIZE + 1) / 2);
        } else {
            assert_eq!(data.len(), C_SIZE);
        }

        Ok(PruningTable {
            entries: data.into_boxed_slice().try_into().unwrap(),
            coord_type: PhantomData,
            source: Loaded,
            table_type
        })
    }

    pub fn get_source(&self) -> TableSource {
        self.source
    }

    pub fn get_type(&self) -> TableType {
        self.table_type
    }

    pub fn get_bytes(&self) -> Vec<u8> {
        let mut ser = vec![VERSION, self.table_type.to_u8().unwrap()];
        ser.extend(self.entries.iter());
        ser
    }

    pub fn get(&self, id: C) -> (u8, u8) {
        let id: usize = id.into();
        match self.table_type {
            TableType::Compressed => {
                let entry = self.entries[id >> 1];
                let val = entry >> ((id & 1) << 2); //branchless entry << (id & 1 == 0 ? 0 : 4)
                (val & 0x0F, 1)
            }
            TableType::Uncompressed => (self.entries[id], 1),
            TableType::Niss => {
                let entry = self.entries[id];
                (entry & 0x0F, entry >> 4)
            }
        }
    }

    pub fn set(&mut self, id: C, entry: u8) {
        let id: usize = id.into();
        match self.table_type {
            TableType::Compressed => {
                let value = self.entries[id >> 1];
                let mask = 0xF0u8 >> ((id & 1) << 2);
                let entry = entry << ((id & 1) << 2);
                self.entries[id >> 1] = value & mask | entry;
            },
            _ => self.entries[id] = entry,
        }
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
    table_type: TableType,
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
    let mut table = PruningTable::new(table_type);
    for (start_coord, start_cube) in visited {
        table.set(start_coord, 0);
        to_check.insert(start_coord, start_cube);
    }
    if to_check.len() > 1 {
        debug!("Found {} variations of the goal state", to_check.len());
    }
    let mut total_checked = 0;
    for depth in 0..20 {
        total_checked += to_check.len();
        debug!(
            "Checked {:width$}/{} cubes at depth {depth} (new {})",
            total_checked,
            CoordParam::size(),
            to_check.len(),
            width = CoordParam::size().to_string().len(),
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
        for m in Transformation::all() {
            let mut cube = cube.clone();
            cube.transform(*m);
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
    table_type: TableType,
) -> PruningTable<COORD_SIZE, CoordParam>
where
    Mapper: Fn(&PuzzleParam) -> CoordParam,
{
    let mut table = PruningTable::new(table_type);
    let mut to_check: HashMap<CoordParam, PuzzleParam> = HashMap::new();
    let start_cube = PuzzleParam::default();
    let start_coord = mapper(&start_cube);
    table.set(start_coord, 0);
    to_check.insert(start_coord, start_cube);
    let mut total_checked = 0;
    for depth in 0..20 {
        total_checked += to_check.len();
        debug!(
            "Checked {:width$}/{} cubes at depth {depth} (new {})",
            total_checked,
            CoordParam::size(),
            to_check.len(),
            width = CoordParam::size().to_string().len(),
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
            let (stored, _) = table.get(coord);
            if stored == table.empty_val() {
                table.set(coord, depth + 1);
                next_cubes.insert(coord, cube);
            }
        }
    }
    next_cubes
}
