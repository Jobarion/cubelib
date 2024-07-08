use std::panic;
use std::str::FromStr;

use leptonic::prelude::*;
use leptos::*;
use log::Level;

use crate::cube::ScrambleComponent;
use crate::solution::SolutionComponent;
use crate::settings::{SettingsComponent, SettingsState};
use crate::step::*;
use crate::util::{build_toggle_chain};
use leptos_icons::IoIcon;

mod cube;
mod step;
mod util;
mod solution;
mod settings;

#[component]
fn App() -> impl IntoView {

    view! {
        <Root default_theme=LeptonicTheme::default()>
            <FMCAppContainer />
        </Root>
    }
}

#[component]
fn FMCAppContainer() -> impl IntoView {

    let (settings_modal, set_settings_modal) = create_signal(false);
    let scramble = util::use_local_storage("scramble", "".to_string());
    provide_context(scramble.clone());

    let enabled_states = build_toggle_chain::<3>("enabled");

    let eo_enabled = create_rw_signal(true);
    let eo = EOConfig::from_local_storage((Signal::derive(move||eo_enabled.get()), Callback::new(move|e|eo_enabled.set(e))));
    let rzp = RZPConfig::from_local_storage();
    let dr = DRConfig::from_local_storage(enabled_states[0]);
    let htr = HTRConfig::from_local_storage(enabled_states[1]);

    let fin = FinishConfig::from_local_storage(enabled_states[2]);

    let fr_signal = util::use_local_storage("enabled-fr", true);

    let enabled_states_1 = enabled_states[1].clone();
    let fr_signal = (Signal::derive(move || fr_signal.0.get() && enabled_states_1.0.get()), Callback::new(move |state| {
        if state {
            enabled_states[0].1.call(true);
            enabled_states[1].1.call(true);
            fr_signal.1.set(true);
        } else {
            fr_signal.1.set(false);
        }
    }));

    let fr = FRConfig::from_local_storage(fr_signal);

    provide_context(eo.clone());
    provide_context(rzp.clone());
    provide_context(dr.clone());
    provide_context(htr.clone());
    provide_context(fr.clone());
    provide_context(fin.clone());

    let settings = SettingsState::from_local_storage();
    provide_context(settings);
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
                    <button
                        on:click=move|_|{
                            scramble.1.set("".to_string());
                            scramble.2();
                            eo.reset();
                            dr.reset();
                            rzp.reset();
                            htr.reset();
                            fr.reset();
                            fin.reset();
                            fin.enabled.1.call(true);
                        }
                        class="icon-button"
                        style:float="right"
                        style:font-size="30px">
                        <Icon icon=IoIcon::IoRefreshOutline/>
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
