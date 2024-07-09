use crate::puzzles::pyraminx::{Pyraminx, PyraminxTransformation, PyraminxTurn};
use crate::solver::moveset::{Transition, TransitionTable};

pub type StepPyraminx<'a> = crate::steps::step::Step<'a, PyraminxTurn, PyraminxTransformation, Pyraminx, TransitionTablePyraminx>;
pub type MoveSetPyraminx = crate::solver::moveset::MoveSet<PyraminxTurn, TransitionTablePyraminx>;

#[derive(Copy, Clone)]
pub struct TransitionTablePyraminx;

impl TransitionTable<PyraminxTurn> for TransitionTablePyraminx {
    fn check_move(&self, _: PyraminxTurn) -> Transition {
        Transition::any()
    }
}

#[cfg(feature = "pyraminxfinish")]
pub mod finish {
    use crate::algs::Algorithm;
    use crate::defs::StepKind;
    use crate::puzzles::puzzle::Invertible;
    use crate::puzzles::pyraminx::{Pyraminx, PyraminxTransformation, PyraminxTurn};
    use crate::puzzles::pyraminx::coords::{NO_TIPS_COORD_SIZE, NoTipsCoord};
    use crate::puzzles::pyraminx::PyraminxTip::{Back, Left, Right, Up};
    use crate::puzzles::pyraminx::steps::{MoveSetPyraminx, StepPyraminx, TransitionTablePyraminx};
    use crate::solver::lookup_table::LookupTable;
    use crate::solver::moveset::MoveSet;
    use crate::steps::coord::ZeroCoord;
    use crate::steps::step::{AnyPostStepCheck, DefaultPruningTableStep, PostStepCheck, PreStepCheck, StepVariant};

    pub type NoTipsFinishStep<'a> = DefaultPruningTableStep::<'a, { NO_TIPS_COORD_SIZE }, NoTipsCoord, 0, ZeroCoord, PyraminxTurn, PyraminxTransformation, Pyraminx, TransitionTablePyraminx, AnyPostStepCheck>;
    pub type NoTipsFinishPruningTable = LookupTable<{ NO_TIPS_COORD_SIZE }, NoTipsCoord>;
    pub type TipsFinishStep<'a> = DefaultPruningTableStep::<'a, { NO_TIPS_COORD_SIZE }, NoTipsCoord, 0, ZeroCoord, PyraminxTurn, PyraminxTransformation, Pyraminx, TransitionTablePyraminx, AnyPostStepCheck>;
    pub type TipsFinishPruningTable = LookupTable<{ NO_TIPS_COORD_SIZE }, NoTipsCoord>;

    pub const NO_TIPS_MOVESET: MoveSetPyraminx = MoveSetPyraminx {
        st_moves: &PyraminxTurn::NO_TIPS,
        aux_moves: &[],
        transitions: &[TransitionTablePyraminx; 16],
    };
    pub const TIPS_ONLY_MOVESET: MoveSetPyraminx = MoveSetPyraminx {
        st_moves: &PyraminxTurn::TIPS,
        aux_moves: &[],
        transitions: &[TransitionTablePyraminx; 16],
    };

    pub fn direct_finish<'a>(table: &'a NoTipsFinishPruningTable) -> StepPyraminx<'a> {
        let step: Box<dyn StepVariant<PyraminxTurn, PyraminxTransformation, Pyraminx, TransitionTablePyraminx> + 'a> = Box::new(NoTipsFinishStep::new(&NO_TIPS_MOVESET, vec![], table, AnyPostStepCheck, ""));
        StepPyraminx::new(vec![step], StepKind::FIN, true)
    }

    pub fn tips<'a>() -> StepPyraminx<'a> {
        let step: Box<dyn StepVariant<PyraminxTurn, PyraminxTransformation, Pyraminx, TransitionTablePyraminx> + 'a> = Box::new(TipStepVariant::new());
        StepPyraminx::new(vec![step], StepKind::Other("TIPS".to_string()), false)
    }

    pub struct TipStepVariant(Vec<PyraminxTransformation>);

    impl TipStepVariant {

        pub fn new() -> TipStepVariant {
            TipStepVariant(vec![])
        }

        const MOVESETS: [MoveSetPyraminx; 8] = [
            MoveSetPyraminx { st_moves: &[PyraminxTurn::u], aux_moves: &[], transitions: &[TransitionTablePyraminx; 16], },
            MoveSetPyraminx { st_moves: &[PyraminxTurn::ui], aux_moves: &[], transitions: &[TransitionTablePyraminx; 16], },
            MoveSetPyraminx { st_moves: &[PyraminxTurn::l], aux_moves: &[], transitions: &[TransitionTablePyraminx; 16], },
            MoveSetPyraminx { st_moves: &[PyraminxTurn::li], aux_moves: &[], transitions: &[TransitionTablePyraminx; 16], },
            MoveSetPyraminx { st_moves: &[PyraminxTurn::r], aux_moves: &[], transitions: &[TransitionTablePyraminx; 16], },
            MoveSetPyraminx { st_moves: &[PyraminxTurn::ri], aux_moves: &[], transitions: &[TransitionTablePyraminx; 16], },
            MoveSetPyraminx { st_moves: &[PyraminxTurn::b], aux_moves: &[], transitions: &[TransitionTablePyraminx; 16], },
            MoveSetPyraminx { st_moves: &[PyraminxTurn::bi], aux_moves: &[], transitions: &[TransitionTablePyraminx; 16], },
        ];
    }

    impl PreStepCheck<PyraminxTurn, PyraminxTransformation, Pyraminx> for TipStepVariant {
        fn is_cube_ready(&self, _: &Pyraminx) -> bool {
            true
        }
    }

    impl PostStepCheck<PyraminxTurn, PyraminxTransformation, Pyraminx> for TipStepVariant {
        fn is_solution_admissible(&self, _: &Pyraminx, _: &Algorithm<PyraminxTurn>) -> bool {
            true
        }
    }

    impl StepVariant<PyraminxTurn, PyraminxTransformation, Pyraminx, TransitionTablePyraminx> for TipStepVariant {
        fn move_set(&self, cube: &Pyraminx, _: u8) -> &'_ MoveSet<PyraminxTurn, TransitionTablePyraminx> {
            let turn = cube.get_tips().iter()
                .zip(vec![Up, Left, Right, Back].into_iter())
                .filter(|(x, _)| x.is_some())
                .map(|(dir, tip)| PyraminxTurn::new(tip, dir.unwrap().invert(), true))
                .next()
                .expect("No moveset if tips are completed");
            &Self::MOVESETS[turn.to_id() >> 1]
        }

        fn pre_step_trans(&self) -> &'_ Vec<PyraminxTransformation> {
            &self.0
        }

        fn heuristic(&self, cube: &Pyraminx, _: u8, _: bool) -> u8 {
            cube.get_tips()
                .iter()
                .filter(|x| x.is_some())
                .count() as u8
        }

        fn name(&self) -> &str {
            ""
        }
    }
}

#[cfg(feature = "pyraminxfinish")]
pub mod tables {
    use std::time::Instant;

    use log::{debug, info};

    use crate::puzzles::pyraminx::coords::NoTipsCoord;
    use crate::puzzles::pyraminx::steps::finish::{NO_TIPS_MOVESET, NoTipsFinishPruningTable};
    #[cfg(feature = "pyraminxfinish")]
    use crate::solver::lookup_table;
    use crate::solver::lookup_table::TableType;

    pub struct PruningTablesPyraminx {
        #[cfg(feature = "pyraminxfinish")]
        direct_finish: Option<NoTipsFinishPruningTable>
    }

    impl PruningTablesPyraminx {

        pub const VERSION: u32 = 1;

        pub fn new() -> PruningTablesPyraminx {
            PruningTablesPyraminx {
                #[cfg(feature = "pyraminxfinish")]
                direct_finish: None
            }
        }

        pub fn load(&mut self, key: &str, data: Box<Vec<u8>>) -> Result<(), String> {
            match key {
                #[cfg(feature = "pyraminxfinish")]
                "fin" => self.direct_finish = Some(NoTipsFinishPruningTable::load(data)?),
                _ => {}
            }
            Ok(())
        }

        #[cfg(feature = "pyraminxfinish")]
        pub fn gen_finish_no_tips(&mut self) {
            if self.direct_finish.is_none() {
                let table = gen_finish_no_tips();
                self.direct_finish = Some(table);
            }
        }

        #[cfg(feature = "pyraminxfinish")]
        pub fn fin(&self) -> Option<&NoTipsFinishPruningTable> {
            self.direct_finish.as_ref()
        }
    }

    #[cfg(feature = "pyraminxfinish")]
    fn gen_finish_no_tips() -> NoTipsFinishPruningTable {
        info!("Generating direct finish pruning table...");
        #[cfg(not(target_arch = "wasm32"))]
            let time = Instant::now();
        let table = lookup_table::generate(&NO_TIPS_MOVESET, &|c: &crate::puzzles::pyraminx::Pyraminx| NoTipsCoord::from(c), TableType::Uncompressed);
        #[cfg(not(target_arch = "wasm32"))]
        debug!("Took {}ms", time.elapsed().as_millis());
        table
    }
}