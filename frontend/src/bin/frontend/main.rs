use std::collections::HashMap;
use std::panic;

use base64::Engine;
use base64::prelude::BASE64_URL_SAFE;
use leptonic::prelude::*;
use leptos::*;
use leptos_icons::IoIcon;
use log::Level;

use crate::cube::ScrambleComponent;
use crate::settings::{SettingsComponent, SettingsState};
use crate::solution::SolutionComponent;
use crate::step::*;
use crate::util::build_toggle_chain;

mod cube;
mod step;
mod util;
mod solution;
mod settings;

#[derive(Clone)]
struct AppContext {
    session: bool
}

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
    load_url_state();
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

    let fr_signal = util::use_local_storage("enabled-fr", false);

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
    let kofi = "<a id='kofi' href='https://ko-fi.com/O4O31AIZTT' target='_blank'><img height='36' style='border:0px;height:36px;' src='https://storage.ko-fi.com/cdn/kofi6.png?v=6' border='0' alt='Buy Me a Coffee at ko-fi.com' /></a>";
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
                    <button
                        on:click=move|_|open_shared()
                        class="icon-button"
                        style:float="right"
                        style:font-size="30px">
                        <Icon icon=IoIcon::IoShareOutline/>
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
            <div style:height="50px" style:position="relative" inner_html=kofi />
        </Box>
    }
}

fn load_url_state() {
    let current_url = url::Url::parse(leptos::window().location().href().unwrap().as_str()).unwrap();
    let mut is_local = current_url.query_pairs()
        .filter(|(k, _)|k == "local")
        .map(|(_, v)|v)
        .next()
        .filter(|v|v == "true")
        .is_some();

    let settings = current_url.query_pairs()
        .filter(|(k, _)|k == "settings")
        .map(|(_, v)|v)
        .next();

    if let Some(settings) = settings {
        is_local = true;
        let decoded = BASE64_URL_SAFE.decode(settings.to_string()).unwrap();
        let decoded = if decoded.len() > 0 && decoded[0] == b'{' {
            String::from_utf8(decoded).unwrap()
        } else {
            let decompressed = miniz_oxide::inflate::decompress_to_vec(&decoded).unwrap();
            String::from_utf8(decompressed).unwrap()
        };
        let settings: HashMap<String, String> = serde_json::from_str(&decoded).unwrap();

        let storage = window().session_storage().unwrap().unwrap();
        let _ = storage.clear();

        for (k, v) in settings.into_iter() {
            let _ = storage.set_item(&k, &v);
        }
    }

    provide_context(AppContext {
        session: is_local
    });
}

fn open_shared() {
    let app_context = use_context::<AppContext>().expect("App context required");
    let storage = if app_context.session {
        window().session_storage().unwrap().unwrap()
    } else {
        window().local_storage().unwrap().unwrap()
    };
    let mut values = HashMap::new();
    for i in 0..storage.length().unwrap() {
        if let Some(key) = storage.key(i).unwrap() {
            if !key.starts_with("mallard-") {
                continue
            }
            if let Some(value) = storage.get_item(&key).unwrap() {
                values.insert(key, value);
            }
        }
    }
    let serialized = serde_json::to_string(&values).unwrap();
    let compressed = miniz_oxide::deflate::compress_to_vec(serialized.as_bytes(), 10);
    let encoded = BASE64_URL_SAFE.encode(compressed);
    let _ = window().open_with_url_and_target(&format!("?local=true&settings={encoded}"), "_blank");
}

fn main() {
    wasm_log::init(wasm_log::Config::new(Level::Debug));
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    leptos::mount_to_body(|| view! {<App/> })
}
