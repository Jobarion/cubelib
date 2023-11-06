use std::collections::HashMap;
use std::str::FromStr;

use cubelib::cube::Axis;
use cubelib::defs::*;
use cubelib::solution::*;
use cubelib::steps::step::StepConfig;
use leptonic::prelude::*;
use leptos::*;

use crate::step::{DRConfig, EOConfig, FinishConfig, FRConfig, HTRConfig, RZPConfig, SelectableAxis};

#[cfg(feature = "backend")]
pub use backend::SolutionComponent;
#[cfg(feature = "wasm_solver")]
pub use wasm_solver::SolutionComponent;


#[cfg(feature = "backend")]
pub mod backend {
    use std::str::FromStr;

    use cubelib::algs::Algorithm;
    use cubelib_interface::{SolverRequest, SolverResponse};
    use leptonic::prelude::*;
    use leptos::*;
    use reqwest::Error;

    use crate::solution::get_step_configs;
    use crate::step::{DRConfig, EOConfig, FinishConfig, FRConfig, HTRConfig, RZPConfig};

    #[component]
    pub fn SolutionComponent() -> impl IntoView {

        let scramble = Signal::derive(move || Algorithm::from_str(use_context::<RwSignal<String>>().expect("Scramble context required").get().as_str()).ok());
        let eo = use_context::<EOConfig>().expect("EO context required");
        let rzp = use_context::<RZPConfig>().expect("RZP context required");
        let dr = use_context::<DRConfig>().expect("DR context required");
        let htr = use_context::<HTRConfig>().expect("HTR context required");
        let fr = use_context::<FRConfig>().expect("FR context required");
        let fin = use_context::<FinishConfig>().expect("Finish context required");

        let request = leptos_use::signal_debounced(Signal::derive(move||{
            if let Some(alg) = scramble.get() {
                let steps = get_step_configs(eo, rzp, dr, htr, fr, fin);
                Some(SolverRequest {
                    steps: steps.clone(),
                    scramble: alg.to_string()
                })
            } else {
                None
            }
        }), 1000.0);

        let solution_resource = create_resource(
            move ||request.get(),
            |req| async move {
                if let Some(req) = req {
                    fetch_solution(req).await.map_err(|err|err.to_string())
                } else {
                    Err("This shouldn't render".to_string())
                }
            });
        view! {
            <Suspense fallback=move || view! {<Code>"Fetching solution..."</Code>}>
                {move|| {
                    let res = solution_resource.read();
                    match res {
                        Some(Ok(res)) => {
                            let solution = res.solution.to_string();
                            view! {
                                <Code>{format!("{solution}")}</Code>
                            }
                        }
                        Some(Err(err)) => view! {
                            <Code>"Error fetching solution"</Code>
                        },
                        None => view! {
                            <Code>"Unknown error"</Code>
                        }
                    }
                }}
            </Suspense>
        }
    }

    async fn fetch_solution(request: SolverRequest) -> Result<SolverResponse, Error> {
        let client = reqwest::Client::new();
        client.post("https://joba.me/cubeapi/solve")
            // client.post("http://localhost:8049/solve")
            .json(&request)
            .send()
            .await?
            .json()
            .await
    }
}

#[cfg(feature = "wasm_solver")]
pub mod wasm_solver {
    use std::collections::HashMap;
    use std::rc::Rc;
    use std::str::FromStr;

    use cubelib::algs::Algorithm;
    use cubelib::cube::{ApplyAlgorithm, Axis, NewSolved};
    use cubelib::cubie::CubieCube;
    use cubelib::defs::*;
    use cubelib::steps::step::StepConfig;
    use gloo_worker::Spawnable;
    use leptonic::prelude::*;
    use leptos::*;
    use leptos_use::watch_debounced;

    use frontend::worker::{FMCSolver, WorkerResponse};
    use crate::solution::get_step_configs;

    use crate::step::{DRConfig, EOConfig, FinishConfig, FRConfig, HTRConfig, RZPConfig, SelectableAxis};

    #[component]
    pub fn SolutionComponent() -> impl IntoView {

        let scramble = Signal::derive(move || Algorithm::from_str(use_context::<RwSignal<String>>().expect("Scramble context required").get().as_str()).ok());
        let eo = use_context::<EOConfig>().expect("EO context required");
        let rzp = use_context::<RZPConfig>().expect("RZP context required");
        let dr = use_context::<DRConfig>().expect("DR context required");
        let htr = use_context::<HTRConfig>().expect("HTR context required");
        let fr = use_context::<FRConfig>().expect("FR context required");
        let fin = use_context::<FinishConfig>().expect("Finish context required");

        let solver_request = Signal::derive(move || {
            let mut cube = CubieCube::new_solved();
            cube.apply_alg(&scramble.get().unwrap_or(Algorithm::new()));
            (cube, get_step_configs(eo, rzp, dr, htr, fr, fin))
        });

        let bridge = FMCSolver::spawner()
            .spawn("./cube/worker.js");

        let bridge = create_rw_signal(Rc::new(bridge));
        let bridge_data = create_rw_signal::<Option<WorkerResponse>>(None);

        watch_debounced(move || solver_request.get(), move |conf, _, _| {
            //TODO check if conf is still valid for the current solution and just skip generating a new one if it is.
            bridge_data.set(None);
            let bridge_handle = FMCSolver::spawner()
                .callback(move |data| {
                    bridge_data.set(Some(data));
                })
                .spawn("./cube/worker.js");
            bridge_handle.send(conf.clone());
            bridge.set(Rc::new(bridge_handle));
        }, 500.0);

        view! {
            {move ||
                match bridge_data.get() {
                    Some(WorkerResponse::Solved(s)) => view! {
                        <Code>{format!("{s}")}</Code>
                    },
                    Some(WorkerResponse::NoSolution) => view! {
                        <Code>{format!("No solution")}</Code>
                    },
                    Some(WorkerResponse::InvalidStepConfig) => view! {
                        <Code>{format!("Invalid step config (this is a bug)")}</Code>
                    },
                    Some(WorkerResponse::UnknownError) => view! {
                        <Code>{format!("Unknown error")}</Code>
                    },
                    None => view! {
                        <Code>"Calculating solution..."</Code>
                    }
                }
            }
        }
    }
}

fn get_step_configs(eo: EOConfig, rzp: RZPConfig, dr: DRConfig, htr: HTRConfig, fr: FRConfig, fin: FinishConfig) -> Vec<StepConfig> {
    let mut steps_config = vec![];
    if eo.enabled.0.get() {
        steps_config.push(StepConfig {
            kind: StepKind::EO,
            substeps: Some(variants_to_string(eo.variants.get())),
            min: Some(eo.min.get()),
            max: Some(eo.max.get()),
            step_limit: None,
            quality: 10000,
            niss: Some(eo.niss.get()),
            params: Default::default(),
        });
    }
    if dr.enabled.0.get() {
        if dr.triggers.get().len() > 0 {
            steps_config.push(StepConfig {
                kind: StepKind::RZP,
                substeps: None,
                min: Some(rzp.min.get()),
                max: Some(rzp.max.get()),
                step_limit: None,
                quality: 10000,
                niss: Some(rzp.niss.get()),
                params: Default::default(),
            });
            let mut triggers = HashMap::new();
            triggers.insert("triggers".to_string(), dr.triggers.get().join(","));
            steps_config.push(StepConfig {
                kind: StepKind::DR,
                substeps: Some(variants_to_string(dr.variants.get())),
                min: Some(dr.min.get()),
                max: Some(dr.max.get()),
                step_limit: None,
                quality: 10000,
                niss: Some(dr.niss.get()),
                params: triggers,
            });
        } else {
            steps_config.push(StepConfig {
                kind: StepKind::DR,
                substeps: Some(variants_to_string(dr.variants.get())),
                min: Some(dr.min.get()),
                max: Some(dr.max.get()),
                step_limit: None,
                quality: 10000,
                niss: Some(dr.niss.get()),
                params: Default::default(),
            });
        }
    }
    if htr.enabled.0.get() {
        steps_config.push(StepConfig {
            kind: StepKind::HTR,
            substeps: Some(variants_to_string(htr.variants.get())),
            min: Some(htr.min.get()),
            max: Some(htr.max.get()),
            step_limit: None,
            quality: 10000,
            niss: Some(htr.niss.get()),
            params: Default::default(),
        });
    }
    if fr.enabled.0.get() {
        steps_config.push(StepConfig {
            kind: if fin.leave_slice.get() {
                StepKind::FRLS
            } else {
                StepKind::FR
            },
            substeps: Some(variants_to_string(fr.variants.get())),
            min: Some(fr.min.get()),
            max: Some(fr.max.get()),
            step_limit: None,
            quality: 10000,
            niss: Some(fr.niss.get()),
            params: Default::default(),
        });
    }
    if fin.enabled.0.get() {
        steps_config.push(StepConfig {
            kind: StepKind::FIN,
            substeps: Some(vec!["ud".to_string(), "fb".to_string(), "lr".to_string()]),
            min: Some(fin.min.get()),
            max: Some(fin.max.get()),
            step_limit: None,
            quality: 10000,
            niss: Some(NissSwitchType::Never),
            params: Default::default(),
        });
    }
    steps_config
}

fn variants_to_string(variants: Vec<Axis>) -> Vec<String> {
    variants.into_iter()
        .map(|a| Into::<SelectableAxis>::into(a).to_string())
        .collect()
}