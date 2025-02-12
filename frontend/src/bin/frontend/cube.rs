use std::str::FromStr;

use cubelib::algs::Algorithm;
use cubelib::puzzles::c333::Cube333;
use cubelib::puzzles::cube::{CubeColor, CubeFace};
use cubelib::puzzles::puzzle::ApplyAlgorithm;
use leptos::*;
use leptonic::prelude::*;
use leptos::html::Div;
use crate::util::RwSignalTup;

#[component]
pub fn ScrambleComponent() -> impl IntoView {
    let scramble = use_context::<RwSignalTup<String>>().unwrap();
    let cube = Signal::derive(move ||{
        Algorithm::from_str(scramble.0.get().as_str()).ok()
            .map(|alg| {
                let mut cube = Cube333::default();
                cube.apply_alg(&alg);
                cube
            })
    });

    view! {
        <div>
            <TextInput get=scramble.0 set=scramble.1 placeholder={"R' U' F".to_owned()}/>
            <Show
                when=move || {cube.get().is_some()}
                fallback=|| view! {<br/><Chip color=ChipColor::Danger>"Invalid scramble"</Chip>}
            >
                <Cube cube=Signal::derive(move ||{
                    cube.get().unwrap_or(Cube333::default())
                })/>
            </Show>
        </div>
    }
}

#[component]
pub fn Cube(cube: Signal<Cube333>) -> impl IntoView {
    let facelets = Signal::derive(move || {
        let facelets = cube.get().get_facelets();

        let mut colors: Vec<CubeColor> = vec![];
        for x in 0..3 {
            colors.append(&mut vec![CubeColor::None; 3]);
            for y in 0..3 {
                colors.push(facelets[CubeFace::Up][x * 3 + y]);
            }
            colors.append(&mut vec![CubeColor::None; 6]);
        }
        for x in 0..3 {
            for face in vec![CubeFace::Left, CubeFace::Front, CubeFace::Right, CubeFace::Back] {
                for y in 0..3 {
                    colors.push(facelets[face][x * 3 + y]);
                }
            }
        }

        for x in 0..3 {
            colors.append(&mut vec![CubeColor::None; 3]);
            for y in 0..3 {
                colors.push(facelets[CubeFace::Down][x * 3 + y]);
            }
            colors.append(&mut vec![CubeColor::None; 6]);
        }
        let html_facelets: Vec<HtmlElement<Div>> = colors.into_iter()
            .map(|c| match c {
                CubeColor::White => Some("cube-facelet-u"),
                CubeColor::Yellow => Some("cube-facelet-d"),
                CubeColor::Green => Some("cube-facelet-f"),
                CubeColor::Blue => Some("cube-facelet-b"),
                CubeColor::Orange => Some("cube-facelet-l"),
                CubeColor::Red => Some("cube-facelet-r"),
                CubeColor::None => None,
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