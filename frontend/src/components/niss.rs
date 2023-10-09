use rand::distributions::Alphanumeric;
use rand::Rng;
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, InputEvent};
use yew::{Callback, Component, function_component, Html, html, NodeRef, Properties, use_state};
use crate::components::DefaultProps;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum NissSwitchType {
    Never,
    Before,
    Always,
}

#[derive(PartialEq, Properties, Clone)]
pub struct NissData {
    pub niss_type: NissSwitchType,
}

#[function_component]
pub fn NissComponent(props: &DefaultProps<NissData>) -> Html {

    let id: String = rand::thread_rng().sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();

    let input = {
        let on_changed = props.on_changed.clone();
        move |e: InputEvent| {
            let target = e.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
            if let Some(input) = input {
                let niss_type = match input.value().as_str() {
                    "none" => NissSwitchType::Never,
                    "before" => NissSwitchType::Before,
                    "always" => NissSwitchType::Always,
                    _ => unreachable!(),
                };
                on_changed.emit(NissData {niss_type});
            }
        }
    };
    html! {
        <fieldset>
            <legend>{"Allow switching to inverse"}</legend>
            <div>
                <input type="radio" value="none" name={format!("{id}")} checked={props.data.niss_type == NissSwitchType::Never} oninput={input.clone()}/>
                <label for="none">{"Never"}</label>
            </div>
            <div>
                <input type="radio" value="before" name={format!("{id}")} checked={props.data.niss_type == NissSwitchType::Before} oninput={input.clone()}/>
                <label for="before">{"Before"}</label>
            </div>
            <div>
                <input type="radio" value="always" name={format!("{id}")} checked={props.data.niss_type == NissSwitchType::Always} oninput={input}/>
                <label for="always">{"Always"}</label>
            </div>
        </fieldset>
    }
}
