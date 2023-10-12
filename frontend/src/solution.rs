use std::collections::HashMap;
use cubelib::algs::{Algorithm, Solution};
use std::str::FromStr;
use cubelib_interface::{NissSwitchType, SolverRequest, SolverResponse, StepConfig, StepKind};
use leptos::*;
use leptonic::prelude::*;
use reqwest::Error;
use crate::step::{DRConfig, EOConfig, FinishConfig, FRConfig, HTRConfig, NissType, RZPConfig, VariantAxis};

#[component]
pub fn SolutionComponent(cx: Scope) -> impl IntoView {

    let scramble = Signal::derive(cx, move || Algorithm::from_str(use_context::<RwSignal<String>>(cx).expect("Scramble context required").get().as_str()).ok());
    let eo = use_context::<EOConfig>(cx).expect("EO context required");
    let rzp = use_context::<RZPConfig>(cx).expect("RZP context required");
    let dr = use_context::<DRConfig>(cx).expect("DR context required");
    let htr = use_context::<HTRConfig>(cx).expect("HTR context required");
    let fr = use_context::<FRConfig>(cx).expect("FR context required");
    let fin = use_context::<FinishConfig>(cx).expect("Finish context required");

    let request = Signal::derive(cx, move||{
        if let Some(alg) = scramble.get() {
            let eo = map_eo_config_to_dto(eo, StepKind::EO);
            let (rzp, dr) = map_rzp_dr_config_to_dtos(rzp, dr);
            let htr = map_htr_config_to_dto(htr, StepKind::HTR);
            let fr = map_fr_config_to_dto(fr, StepKind::FR);
            let fin = map_fin_config_to_dto(fin);


            let steps: Vec<StepConfig> = vec![eo, rzp, dr, htr, fr, fin].into_iter()
                .flat_map(|f|f)
                .collect();
            Some(SolverRequest {
                quality: Some(1000),
                steps: steps.clone(),
                scramble: alg.to_string()
            })
        } else {
            None
        }
    });


    let solution_resource = create_resource(
        cx,
        move ||request.get(),
        |req| async move {
            if let Some(req) = req {
                fetch_solution(req).await.map_err(|err|err.to_string())
            } else {
                Err("This shouldn't render".to_string())
            }
    });
    view! {cx,
        <Suspense fallback=move || view! {cx, <Code>"Fetching solution..."</Code>}>
            {move|| {
                let res = solution_resource.read(cx);
                match res {
                    Some(Ok(res)) => {
                        let solution: Solution = res.into();
                        view! {cx,
                            <Code>{format!("{solution}")}</Code>
                        }
                    }
                    Some(Err(err)) => view! {cx,
                        <Code>"Error fetching solution"</Code>
                    },
                    None => view! {cx,
                        <Code>"Unknown error"</Code>
                    }
                }
            }}
        </Suspense>
    }
}

#[component]
pub fn SolutionSteps(cx: Scope, solution: Solution) -> impl IntoView {

}

async fn fetch_solution(request: SolverRequest) -> Result<SolverResponse, Error> {
    let client = reqwest::Client::new();
    client.post("http://localhost:8081/solve")
        .json(&request)
        .send()
        .await?
        .json()
        .await
}

//TODO fix this mess
fn map_eo_config_to_dto(config: EOConfig, kind: StepKind) -> Option<StepConfig> {
    if !config.enabled.0.get() {
        None
    } else {
        Some(StepConfig {
            kind: kind,
            substeps: config.variants.get().into_iter().map(map_substep).collect(),
            min: config.min.get(),
            max: config.max.get(),
            niss: map_niss_type(config.niss.get()),
            params: Default::default(),
        })
    }
}

fn map_htr_config_to_dto(config: HTRConfig, kind: StepKind) -> Option<StepConfig> {
    if !config.enabled.0.get() {
        None
    } else {
        Some(StepConfig {
            kind: kind,
            substeps: config.variants.get().into_iter().map(map_substep).collect(),
            min: config.min.get(),
            max: config.max.get(),
            niss: map_niss_type(config.niss.get()),
            params: Default::default(),
        })
    }
}

fn map_fr_config_to_dto(config: FRConfig, kind: StepKind) -> Option<StepConfig> {
    if !config.enabled.0.get() {
        None
    } else {
        Some(StepConfig {
            kind: kind,
            substeps: config.variants.get().into_iter().map(map_substep).collect(),
            min: config.min.get(),
            max: config.max.get(),
            niss: map_niss_type(config.niss.get()),
            params: Default::default(),
        })
    }
}

fn map_fin_config_to_dto(config: FinishConfig) -> Option<StepConfig> {
    if !config.enabled.0.get() {
        None
    } else {
        Some(StepConfig {
            kind: StepKind::FIN,
            substeps: vec!["ud".to_string(), "fb".to_string(), "lr".to_string()],
            min: config.min.get(),
            max: config.max.get(),
            niss: NissSwitchType::Never,
            params: Default::default(),
        })
    }
}

fn map_rzp_config_to_dto(config: RZPConfig) -> StepConfig {
    StepConfig {
        kind: StepKind::RZP,
        substeps: vec![],
        min: config.min.get(),
        max: config.max.get(),
        niss: map_niss_type(config.niss.get()),
        params: Default::default(),
    }
}

fn map_rzp_dr_config_to_dtos(rzp: RZPConfig, dr: DRConfig) -> (Option<StepConfig>, Option<StepConfig>) {
    if !dr.enabled.0.get() {
        (None, None)
    } else if dr.triggers.get().is_empty() {
        (None,
         Some(StepConfig {
             kind: StepKind::DR,
             substeps: dr.variants.get().into_iter().map(map_substep).collect(),
             min: dr.min.get(),
             max: dr.max.get(),
             niss: map_niss_type(dr.niss.get()),
             params: Default::default(),
         })
        )
    } else {
        let mut props = HashMap::default();
        props.insert("triggers".to_string(), dr.triggers.get().join(",").replace(" ", ""));
        (Some(StepConfig {
            kind: StepKind::RZP,
            substeps: vec![],
            min: rzp.min.get(),
            max: rzp.max.get(),
            niss: map_niss_type(rzp.niss.get()),
            params: Default::default(),
         }),
         Some(StepConfig {
             kind: StepKind::DR,
             substeps: dr.variants.get().into_iter().map(map_substep).collect(),
             min: dr.min.get(),
             max: dr.max.get(),
             niss: map_niss_type(dr.niss.get()),
             params: props,
         })
        )
    }
}

fn map_niss_type(niss: NissType) -> NissSwitchType {
    match niss {
        NissType::Never => NissSwitchType::Never,
        NissType::Before => NissSwitchType::Before,
        NissType::Always => NissSwitchType::Always,
    }
}

fn map_substep(axis: VariantAxis) -> String {
    match axis {
        VariantAxis::UD => "ud",
        VariantAxis::FB => "fb",
        VariantAxis::LR => "lr",
    }.to_string()
}