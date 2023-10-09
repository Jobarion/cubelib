use cubelib::cube::{Color, Cube, Face};
use cubelib::cubie::CubieCube;
use leptos::*;
use leptos::html::Div;

#[component]
pub fn Cube(cx: Scope, cube: ReadSignal<CubieCube>) -> impl IntoView {
    let facelets = Signal::derive(cx, move || {
        let facelets = cube.get().get_facelets();

        let mut colors: Vec<Color> = vec![];
        for x in (0..3).rev() {
            colors.append(&mut vec![Color::None; 3]);
            for y in (0..3).rev() {
                colors.push(facelets[Face::Back][x * 3 + y]);
            }
            colors.append(&mut vec![Color::None; 6]);
        }
        for x in 0..3 {
            let x_rev = 2 - x;
            for y in (0..3).rev() {
                colors.push(facelets[Face::Left][x + y * 3]);
            }
            for y in 0..3 {
                colors.push(facelets[Face::Up][x * 3 + y]);
            }
            for y in 0..3 {
                colors.push(facelets[Face::Right][x_rev + y * 3]);
            }
            for y in (0..3).rev() {
                colors.push(facelets[Face::Down][x_rev * 3 + y]);
            }
        }

        for x in 0..3 {
            colors.append(&mut vec![Color::None; 3]);
            for y in 0..3 {
                colors.push(facelets[Face::Front][x * 3 + y]);
            }
            colors.append(&mut vec![Color::None; 6]);
        }
        let html_facelets: Vec<HtmlElement<Div>> = colors.into_iter()
            .map(|c| match c {
                Color::White => Some("cube-facelet-u"),
                Color::Yellow => Some("cube-facelet-d"),
                Color::Green => Some("cube-facelet-f"),
                Color::Blue => Some("cube-facelet-b"),
                Color::Orange => Some("cube-facelet-l"),
                Color::Red => Some("cube-facelet-r"),
                Color::None => None,
            })
            .map(|class| class.map_or(
                view! {cx, <div class="cube-facelet"></div> },
                |c| view! {cx, <div class={format!("cube-facelet {c}")}></div> }))
            .collect();
        html_facelets
    });
    view! {cx,
        <div class="cube-expanded-view">
            {move || facelets.get()}
        </div>
    }
}