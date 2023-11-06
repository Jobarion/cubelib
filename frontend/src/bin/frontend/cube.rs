use std::str::FromStr;

use cubelib::algs::Algorithm;
use cubelib::cube::{ApplyAlgorithm, Color, Cube, Face, NewSolved};
use cubelib::cubie::CubieCube;
use leptos::*;
use leptonic::prelude::*;
use leptos::html::Div;

#[component]
pub fn ScrambleComponent() -> impl IntoView {
    let scramble = use_context::<RwSignal<String>>().unwrap();
    let cube = Signal::derive(move ||{
        Algorithm::from_str(scramble.get().as_str()).ok()
            .map(|alg| {
                let mut cube = CubieCube::new_solved();
                cube.apply_alg(&alg);
                cube
            })
    });

    view! {
        <div>
            <TextInput get=scramble set=Callback::new(move|s|scramble.set(s)) placeholder={"R' U' F".to_owned()}/>
            <Show
                when=move || {cube.get().is_some()}
                fallback=|| view! {<br/><Chip color=ChipColor::Danger>"Invalid scramble"</Chip>}
            >
                <Cube cube=Signal::derive(move ||{
                    cube.get().unwrap_or(CubieCube::new_solved())
                })/>
            </Show>
        </div>
    }
}

#[component]
pub fn Cube(cube: Signal<CubieCube>) -> impl IntoView {
    let facelets = Signal::derive(move || {
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
                view! {<div class="cube-facelet"></div> },
                |c| view! {<div class={format!("cube-facelet {c}")}></div> }))
            .collect();
        html_facelets
    });
    view! {
        <div class="cube-expanded-view">
            {move || facelets.get()}
        </div>
    }
}