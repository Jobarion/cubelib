use std::panic;
use std::str::FromStr;

use leptonic::prelude::*;
use leptos::*;
use log::Level;

use crate::cube::ScrambleComponent;
use crate::solution::SolutionComponent;
use crate::settings::{SettingsComponent, SettingsState};
use crate::step::*;
use crate::util::build_toggle_chain;
use leptos_icons::IoIcon;
use leptos_use::storage::use_local_storage;

mod cube;
mod step;
mod util;
mod solution;
mod settings;

#[component]
fn App() -> impl IntoView {
    let scramble = create_rw_signal("".to_string());
    provide_context(scramble);

    let enabled_states = build_toggle_chain::<4>();

    let eo_enabled = create_rw_signal(true);
    let eo = EOConfig::from_local_storage((Signal::derive(move||eo_enabled.get()), Callback::new(move|e|eo_enabled.set(e))));
    let rzp = RZPConfig::from_local_storage();
    let dr = DRConfig::from_local_storage(enabled_states[0]);
    let htr = HTRConfig::from_local_storage(enabled_states[1]);
    let fr = FRConfig::from_local_storage(enabled_states[2]);
    let fin = FinishConfig::from_local_storage(enabled_states[3]);

    provide_context(eo);
    provide_context(rzp);
    provide_context(dr);
    provide_context(htr);
    provide_context(fr);
    provide_context(fin);

    let settings = SettingsState::from_local_storage();
    provide_context(settings);

    view! {
        <Root default_theme=LeptonicTheme::default()>
            <FMCAppContainer />
        </Root>
    }
}

#[component]
fn FMCAppContainer() -> impl IntoView {

    let (settings_modal, set_settings_modal) = create_signal(false);

    view! {
        <Box id="app-container">
            <div>
                <h2>
                    <span style:float="left">"Scramble"</span>
                    <button
                        on:click=move|_|set_settings_modal.set(true)
                        class="icon-button"
                        style:float="right"
                        style:font-size="30px">
                        <Icon icon=IoIcon::IoSettingsOutline/>
                    </button>
                    <div style:clear="both"></div>
                </h2>
            </div>
            <ScrambleComponent/>
            <h2>"Steps"</h2>
            <StepsComponent/>
            <h2>"Solution"</h2>
            <SolutionComponent/>
            <SettingsComponent active=settings_modal set_active=set_settings_modal/>
        </Box>
    }
}

fn main() {
    wasm_log::init(wasm_log::Config::new(Level::Debug));
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    leptos::mount_to_body(|| view! {<App/> })
}
