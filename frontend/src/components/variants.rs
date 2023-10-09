use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, InputEvent};
use yew::{Callback, Component, function_component, Html, html, NodeRef, Properties, use_state};
use crate::components::DefaultProps;

#[derive(PartialEq, Properties, Clone)]
pub struct AxisVariantData {
    pub ud: bool,
    pub fb: bool,
    pub lr: bool,
}

#[function_component]
pub fn AxisVariantComponent(props: &DefaultProps<AxisVariantData>) -> Html {
    let refs_handle = use_state(||(NodeRef::default(), NodeRef::default(), NodeRef::default()));
    let refs = (*refs_handle).clone();

    let input = {
        let on_changed = props.on_changed.clone();
        let r0 = refs.0.clone();
        let r1 = refs.1.clone();
        let r2 = refs.2.clone();
        move |e: InputEvent| {
            let target = e.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
            if let Some(input) = input {
                let ud = r0.cast::<HtmlInputElement>()
                    .expect("ud ref not attached")
                    .checked();
                let fb = r1.cast::<HtmlInputElement>()
                    .expect("fb ref not attached")
                    .checked();
                let lr = r2.cast::<HtmlInputElement>()
                    .expect("lr ref not attached")
                    .checked();
                on_changed.emit(AxisVariantData { ud, fb, lr });
            }
        }
    };
    html! {
        <fieldset>
            <legend>{"Variants"}</legend>
            <div>
                <input type="checkbox" value="ud" ref={refs.0} checked={props.data.ud} oninput={input.clone()}/>
                <label for="ud">{"UD"}</label>
            </div>
            <div>
                <input type="checkbox" value="fb" ref={refs.1}  checked={props.data.fb} oninput={input.clone()}/>
                <label for="fb">{"FB"}</label>
            </div>
            <div>
                <input type="checkbox" value="lr" ref={refs.2}  checked={props.data.lr} oninput={input}/>
                <label for="lr">{"LR"}</label>
            </div>
        </fieldset>
    }
}
