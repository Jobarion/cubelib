pub type Step222<'a> = crate::steps::step::Step<'a, crate::puzzles::c222::Turn222, crate::puzzles::c222::Transformation222, crate::puzzles::c222::Cube222, crate::solver::moveset::TransitionTable222>;
pub type MoveSet222 = crate::solver::moveset::MoveSet<crate::puzzles::c222::Turn222, crate::solver::moveset::TransitionTable222>;

#[cfg(feature = "222finish")]
pub mod finish {
    use crate::defs::StepKind;
    use crate::puzzles::c222::{Cube222, Transformation222, Turn222};
    use crate::puzzles::c222::coords::{CORNER_COORD_SIZE, CornerCoord};
    use crate::puzzles::c222::steps::{MoveSet222, Step222};
    use crate::solver::lookup_table::PruningTable;
    use crate::solver::moveset::TransitionTable222;
    use crate::steps::coord::ZeroCoord;
    use crate::steps::step::{AnyPostStepCheck, DefaultPruningTableStep, StepVariant};

    pub type DirectFinishPruningTableStep<'a> = DefaultPruningTableStep::<'a, {CORNER_COORD_SIZE}, CornerCoord, 0, ZeroCoord, Turn222, Transformation222, Cube222, TransitionTable222, AnyPostStepCheck>;
    pub type DirectFinishPruningTable = PruningTable<{CORNER_COORD_SIZE}, CornerCoord>;
    pub const DIRECT_FINISH_MOVESET: MoveSet222 = MoveSet222 {
        st_moves: &Turn222::ALL,
        aux_moves: &[],
        transitions: &TransitionTable222::DEFAULT_ALL,
    };

    pub fn direct_finish<'a>(table: &'a DirectFinishPruningTable) -> Step222<'a> {
        let step: Box<dyn StepVariant<Turn222, Transformation222, Cube222, TransitionTable222> + 'a> = Box::new(DirectFinishPruningTableStep::new(&DIRECT_FINISH_MOVESET, vec![], table, AnyPostStepCheck, ""));
        Step222::new(vec![step], StepKind::FIN, true)
    }
}

#[cfg(any(feature = "222finish"))]
pub mod tables {
    use std::time::Instant;

    use log::{debug, info};
    use crate::puzzles::c222::coords::CornerCoord;

    #[cfg(feature = "222finish")]
    use crate::puzzles::c222::steps::finish::{DIRECT_FINISH_MOVESET, DirectFinishPruningTable};
    use crate::solver::lookup_table;

    pub struct PruningTables222 {
        #[cfg(feature = "222finish")]
        direct_finish: Option<DirectFinishPruningTable>
    }

    impl PruningTables222 {

        pub const VERSION: u32 = 1;

        pub fn new() -> PruningTables222 {
            PruningTables222 {
                #[cfg(feature = "222finish")]
                direct_finish: None
            }
        }

        pub fn load(&mut self, key: &str, data: Vec<u8>) {
            match key {
                #[cfg(feature = "222finish")]
                "fin" => self.direct_finish = Some(DirectFinishPruningTable::load(data)),
                _ => {}
            }
        }

        #[cfg(feature = "222finish")]
        pub fn gen_fin(&mut self) {
            if self.direct_finish.is_none() {
                let table = gen_direct_finish();
                self.direct_finish = Some(table);
            }
        }

        #[cfg(feature = "222finish")]
        pub fn fin(&self) -> Option<&DirectFinishPruningTable> {
            self.direct_finish.as_ref()
        }
    }

    #[cfg(feature = "222finish")]
    fn gen_direct_finish() -> DirectFinishPruningTable {
        info!("Generating direct finish pruning table...");
        #[cfg(not(target_arch = "wasm32"))]
            let time = Instant::now();
        let table = lookup_table::generate(&DIRECT_FINISH_MOVESET, &|c: &crate::puzzles::c222::Cube222| CornerCoord::from(c));
        #[cfg(not(target_arch = "wasm32"))]
        debug!("Took {}ms", time.elapsed().as_millis());
        table
    }
}