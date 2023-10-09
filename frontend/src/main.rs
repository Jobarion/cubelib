use leptos::*;
use leptos::WriteSignal;
use wasm_bindgen::prelude::wasm_bindgen;

#[component]
fn App() -> impl IntoView {
    let (count, set_count): (ReadSignal<u32>, WriteSignal<u32>) = create_signal(0);

    let view = view! {
        <input type="range" min="0" max="6"></input>
    };
    // web_sys::console::log_1(&format!("{:?}", view.get_attribute("id")).into());
    // create_slider("slider", 1, 10, 3, 5);
    // view.

    view
}

fn main() {
    leptos::mount_to_body(|| view! { <App/> })
}

#[wasm_bindgen(module = "/src/js/slider.js")]
extern "C" {
    fn create_slider(id: &str, min: u32, max: u32, min_selected: u32, max_selected: u32);
    fn set_slider(id: &str, min: u32, max: u32);
}
