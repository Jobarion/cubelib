use std::str::FromStr;
use cubelib::algs::Algorithm;

use cubelib::cube::{ApplyAlgorithm, NewSolved};
use cubelib::cubie::CubieCube;
use leptonic::prelude::*;
use leptos::*;

use crate::cube::Cube;
use crate::cube::ScrambleComponent;
use crate::step::DefaultStepParameters;
use crate::solution::SolutionComponent;
use crate::step::*;
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
    leptos::mount_to_body(|| view! {<App/> })
}
