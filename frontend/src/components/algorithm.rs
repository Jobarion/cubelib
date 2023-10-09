use std::str::FromStr;

use cubelib::algs::Algorithm;
use cubelib::cube::{ApplyAlgorithm, Cube, NewSolved};
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, InputEvent};
use yew::{Callback, function_component, Html, html, Properties, use_state};

#[derive(PartialEq, Properties, Clone)]
pub struct AlgorithmInputProps {
    pub on_changed: Callback<(String, Option<Algorithm>)>,
    pub alg: String,
}

#[function_component]
pub fn AlgorithmInputComponent(props: &AlgorithmInputProps) -> Html {
    let alg_state_handle = use_state(||Some(Algorithm::new()));
    let alg_state = (*alg_state_handle).clone();

    let input = {
        let on_changed = props.on_changed.clone();
        move |e: InputEvent| {
            let target = e.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
            if let Some(input) = input {
                let alg_raw = input.value();
                let alg = Algorithm::from_str(alg_raw.as_str()).ok();
                alg_state_handle.set(alg.clone());
                on_changed.emit((alg_raw, alg));
            }
        }
    };

    html! {
        <div>
            <input oninput={input} value={props.alg.clone()}/>
            <p hidden={alg_state.is_some()}>{"Invalid scramble"}</p>
        </div>
    }
}