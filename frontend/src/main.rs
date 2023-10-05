use cubelib::cube::{Move, NewSolved, Turnable};
use yew::prelude::*;
use cubelib::cubie::CubieCube;

#[function_component]
fn App() -> Html {
    let mut cube = CubieCube::new_solved();
    cube.turn(Move::R);
    cube.turn(Move::U);
    cube.turn(Move::Ri);
    cube.turn(Move::Ui);
    let first_string = cube.to_string();

    let counter = use_state(|| 0);
    let onclick = {
        let counter = counter.clone();
        move |_| {
            let value = *counter + 1;
            counter.set(value);
        }
    };

    html! {
        <div>
            <input onchange={onchange}/>
            <button {onclick}>{ "+1" }</button>
            <p>{ *counter }</p>
            <div style="white-space: pre-wrap;">{ first_string }</div>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}