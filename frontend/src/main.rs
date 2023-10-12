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
fn App(cx: Scope) -> impl IntoView {
    let scramble = create_rw_signal(cx, "".to_string());
    provide_context(cx, scramble);

    let enabled_states = build_toggle_chain::<4>(cx);

    let eo_enabled = create_rw_signal(cx, true);
    let eo = EOConfig::new(cx, (Signal::derive(cx, move||eo_enabled.get()), Callback::new(cx, move|e|eo_enabled.set(e))));
    let rzp = RZPConfig::new(cx);
    let dr = DRConfig::new(cx, enabled_states[0]);
    let htr = HTRConfig::new(cx, enabled_states[1]);
    let fr = FRConfig::new(cx, enabled_states[2]);
    let fin = FinishConfig::new(cx, enabled_states[3]);

    provide_context(cx, eo);
    provide_context(cx, rzp);
    provide_context(cx, dr);
    provide_context(cx, htr);
    provide_context(cx, fr);
    provide_context(cx, fin);

    view! {cx,
        <Root default_theme=LeptonicTheme::default()>
            <FMCAppContainer />
        </Root>
    }
}

#[component]
fn FMCAppContainer(cx: Scope) -> impl IntoView {

    view! {cx,
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
    leptos::mount_to_body(|cx| view! {cx, <App/> })
}
