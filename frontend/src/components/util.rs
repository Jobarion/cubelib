use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, InputEvent};
use yew::{Callback, Component, function_component, Html, html, Properties};

#[derive(PartialEq, Properties, Clone)]
pub struct AxisVariantProps {
    pub ud: bool,
    pub fb: bool,
    pub lr: bool,
    pub on_changed: Callback<(bool, bool, bool)>
}

#[function_component]
pub fn AxisVariantComponent(props: &AxisVariantProps) -> Html {
    let
    let input = {
        let on_changed = props.on_changed.clone();
        move |e: InputEvent| {
            let target = e.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
            if let Some(input) = input {
                let checked = input.value();
                match checked {

                }
                web_sys::console::log_1(&format!("{checked}").into());
                on_changed.emit((true, true, false));
            }
        }
    };
    html! {
        <fieldset>
            <legend>{"Select variants"}</legend>
            <div>
                <input type="checkbox" value="ud" checked={props.ud} oninput={input.clone()}/>
                <label for="ud">{"UD"}</label>
            </div>
            <div>
                <input type="checkbox" value="fb" checked={props.fb} oninput={input.clone()}/>
                <label for="fb">{"FB"}</label>
            </div>
            <div>
                <input type="checkbox" value="lr" checked={props.lr} oninput={input}/>
                <label for="lr">{"LR"}</label>
            </div>
        </fieldset>
    }
}
