mod cube;

use cubelib::cubie::CubieCube;
use cubelib::cube::NewSolved;
use leptos::*;
use leptonic::prelude::*;
use crate::cube::Cube;


#[component]
fn App(cx: Scope) -> impl IntoView {
    let (value_a, set_value_a) = create_signal(cx, 0.5);
    let (value_b, set_value_b) = create_signal(cx, 0.75);
    let (cube, set_cube) = create_signal(cx, CubieCube::new_solved());
    view! {cx,
        <Root default_theme=LeptonicTheme::default()>
            <RangeSlider
                value_a=value_a
                value_b=value_b
                set_value_a=set_value_a
                set_value_b=set_value_b
                min=0.0
                max=10.0
                step=1.0
                popover=SliderPopover::Always
                marks=SliderMarks::Automatic { create_names: false }
                value_display=create_callback(cx, move |v| format!("{v:.0}"))
            />
            <Cube cube=cube/>
        </Root>
    }
}

fn main() {
    leptos::mount_to_body(|cx| view! {cx, <App/> })
}
