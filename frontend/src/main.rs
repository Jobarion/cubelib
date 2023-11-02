use std::panic;
use std::str::FromStr;

use cubelib::algs::Algorithm;
use cubelib::cube::{ApplyAlgorithm, Axis, Move, NewSolved, Turnable};
use cubelib::cubie::CubieCube;
use cubelib::steps::dr::coords::DRUDEOFBCoord;
use cubelib::steps::dr::dr_trigger_config::dr;
use cubelib::steps::eo::coords::EOCoordUD;
use cubelib::steps::step::{DefaultStepOptions, first_step, StepConfig};
use leptonic::prelude::*;
use leptos::*;
use log::{info, Level};

use crate::cube::Cube;
use crate::cube::ScrambleComponent;
use crate::solution::SolutionComponent;
use crate::step::*;
use crate::step::DefaultStepParameters;
use crate::util::build_toggle_chain;

mod cube;
mod step;
mod util;
mod solution;

#[component]
fn App() -> impl IntoView {
    let scramble = create_rw_signal("".to_string());
    provide_context(scramble);

    let enabled_states = build_toggle_chain::<4>();

    let eo_enabled = create_rw_signal(true);
    let eo = EOConfig::new((Signal::derive(move||eo_enabled.get()), Callback::new(move|e|eo_enabled.set(e))));
    let rzp = RZPConfig::new();
    let dr = DRConfig::new(enabled_states[0]);
    let htr = HTRConfig::new(enabled_states[1]);
    let fr = FRConfig::new(enabled_states[2]);
    let fin = FinishConfig::new(enabled_states[3]);

    provide_context(eo);
    provide_context(rzp);
    provide_context(dr);
    provide_context(htr);
    provide_context(fr);
    provide_context(fin);

    view! {
        <Root default_theme=LeptonicTheme::default()>
            <FMCAppContainer />
        </Root>
    }
}

#[component]
fn FMCAppContainer() -> impl IntoView {
    view! {
        <Box id="app-container">
            <h2>"Scramble"</h2>
            <ScrambleComponent/>
            <h2>"Steps"</h2>
            <StepsComponent/>
            <h2>"Solution"</h2>
            <SolutionComponent/>
        </Box>
    }
}

fn main() {
    wasm_log::init(wasm_log::Config::new(Level::Debug));
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let mut pt = cubelib::tables::PruningTables::new();
    pt.gen_eo();
    pt.gen_dr();
    pt.gen_htr();
    pt.gen_fr();
    pt.gen_fr_finish();

    let eo_step = (
        cubelib::steps::eo::eo_config::eo(&pt.eo().unwrap(), vec![Axis::UD, Axis::FB, Axis::LR]),
        DefaultStepOptions {
            niss_type: cubelib::defs::NissSwitchType::Never,
            min_moves: 0,
            max_moves: 5,
            step_limit: Some(10000)
        }
    );
    let dr_step = (
        cubelib::steps::dr::dr_config::dr(&pt.dr().unwrap(), [Axis::UD, Axis::FB, Axis::LR], [Axis::UD, Axis::FB, Axis::LR]),
        DefaultStepOptions {
            niss_type: cubelib::defs::NissSwitchType::Before,
            min_moves: 0,
            max_moves: 12,
            step_limit: Some(10000)
        }
    );
    let htr_step = (
        cubelib::steps::htr::htr_config::htr(&pt.htr().unwrap(), vec![Axis::UD, Axis::FB, Axis::LR]),
        DefaultStepOptions {
            niss_type: cubelib::defs::NissSwitchType::Before,
            min_moves: 0,
            max_moves: 12,
            step_limit: Some(10000)
        }
    );
    let fr_step = (
        cubelib::steps::fr::fr_config::fr(&pt.fr().unwrap(), vec![Axis::UD, Axis::FB, Axis::LR]),
        DefaultStepOptions {
            niss_type: cubelib::defs::NissSwitchType::Before,
            min_moves: 0,
            max_moves: 12,
            step_limit: Some(10000)
        }
    );
    let finish_step = (
        cubelib::steps::finish::finish_config::fr_finish(&pt.fr_finish().unwrap(), vec![Axis::UD, Axis::FB, Axis::LR]),
        DefaultStepOptions {
            niss_type: cubelib::defs::NissSwitchType::Before,
            min_moves: 0,
            max_moves: 12,
            step_limit: Some(10000)
        }
    );
    let steps = vec![eo_step, dr_step, htr_step, fr_step, finish_step];

    let mut cube = CubieCube::new_solved();
    cube.turn(Move::Ri);
    cube.turn(Move::Ui);
    cube.turn(Move::F);
    cube.turn(Move::D2);
    cube.turn(Move::L2);
    cube.turn(Move::F2);
    cube.turn(Move::R);
    cube.turn(Move::B2);
    cube.turn(Move::Ri);
    cube.turn(Move::Ui);
    cube.turn(Move::F);

    let mut sol = cubelib::solver::solve_steps(cube, &steps);
    log::info!("Solving now");
    web_sys::console::log_1(&format!("{}", sol.next().unwrap().to_string()).into());

    leptos::mount_to_body(|| view! {<App/> })
}
