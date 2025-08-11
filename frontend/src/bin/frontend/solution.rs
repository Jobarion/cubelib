use std::collections::{HashMap, HashSet};
use cubelib::algs::Algorithm;
use cubelib::defs::*;
use cubelib::cube::*;
use cubelib::steps::step::StepConfig;
use leptos::*;

#[cfg(feature = "backend")]
pub use backend::SolutionComponent;
#[cfg(feature = "wasm_solver")]
pub use wasm_solver::SolutionComponent;

use crate::settings::SettingsState;
use crate::step::{DRConfig, EOConfig, FinishConfig, FRConfig, HTRConfig, RZPConfig, SelectableAxis};

#[cfg(feature = "backend")]
pub mod backend {
    use std::cell::RefCell;
    use std::collections::{HashMap, HashSet};
    use std::str::FromStr;

    use cubelib::algs::Algorithm;
    use cubelib::defs::StepKind;
    use cubelib::solver::solution::{Solution, SolutionStep};
    use cubelib_interface::{SolverRequest, SolverResponse};
    use ehttp::Request;
    use leptonic::Out::WriteSignal;
    use leptonic::prelude::*;
    use leptos::*;
    use leptos_icons::IoIcon;
    use leptos_use::{watch_debounced_with_options, WatchDebouncedOptions};

    use crate::settings::SettingsState;
    use crate::solution::get_step_configs;
    use crate::step::{DRConfig, EOConfig, FinishConfig, FRConfig, HTRConfig, RZPConfig};
    use crate::util::RwSignalTup;

    #[derive(Clone)]
    enum SolutionState {
        NotFetched,
        Requested,
        Found(ehttp::Result<Solution>)
    }

    #[component]
    pub fn SolutionComponent() -> impl IntoView {
        let scramble = Signal::derive(move || Algorithm::from_str(use_context::<RwSignalTup<String>>().expect("Scramble context required").0.get().as_str()).ok());
        let eo = use_context::<EOConfig>().expect("EO context required");
        let rzp = use_context::<RZPConfig>().expect("RZP context required");
        let dr = use_context::<DRConfig>().expect("DR context required");
        let htr = use_context::<HTRConfig>().expect("HTR context required");
        let fr = use_context::<FRConfig>().expect("FR context required");
        let fin = use_context::<FinishConfig>().expect("Finish context required");
        let settings = use_context::<SettingsState>().expect("Settings required");

        let settings1 = settings.clone();
        let req_signal = {
            let eo = eo.clone();
            let rzp = rzp.clone();
            let dr = dr.clone();
            let htr = htr.clone();
            let fr = fr.clone();
            let fin = fin.clone();
            Signal::derive(move||{
                if let Some(alg) = scramble.get() {
                    let steps = get_step_configs(eo.clone(), rzp.clone(), dr.clone(), htr.clone(), fr.clone(), fin.clone(), &settings);
                    Some(SolverRequest {
                        steps: steps.clone(),
                        scramble: alg.to_string(),
                    })
                } else {
                    None
                }
            })
        };

        let prev_req = create_rw_signal::<Option<Option<SolverRequest>>>(None);

        let solution_data = create_rw_signal(SolutionState::NotFetched);
        let is_done_data = create_rw_signal(true);
        let req_id = create_rw_signal(0usize);

        let _ = watch_debounced_with_options(move || (req_signal.get()), move |req, _, _| {
            let req = req.clone();
            //watch_debounced previous is buggy so we do this
            if let Some(prev) = prev_req.get() {
                if prev == req {
                    return;
                }
            }
            prev_req.set(Some(req.clone()));
            if let Some(req) = req {
                req_id.update(|x| *x = *x + 1);
                if req.scramble.is_empty() {
                    solution_data.set(SolutionState::NotFetched);
                    return;
                }
                solution_data.set(SolutionState::Requested);
                is_done_data.set(false);
                fetch_solution(req.clone(), req_id.get(), solution_data, is_done_data, req_id);
            }
        }, 1000f64, WatchDebouncedOptions::default().immediate(true));
        let eo_step = find_step(solution_data.into(), StepKind::EO);
        let dr_step = find_step(solution_data.into(), StepKind::DR);
        let htr_step = find_step(solution_data.into(), StepKind::HTR);
        let fr_step = find_step(solution_data.into(), StepKind::FR);
        let fin_step = find_step(solution_data.into(), StepKind::FIN);

        view! {
            {move ||
                match solution_data.get() {
                    SolutionState::Found(Ok(s)) => view! {
                        <Code>{format!("{}", s)}</Code>
                    }.into_view(),
                    SolutionState::Found(Err(err)) => view! {
                        <Code>{format!("Error fetching request: {}", err)}</Code>
                    }.into_view(),
                    SolutionState::Requested if is_done_data.get() => view! { //Kind of a hack :(
                        <Code>"No solution found"</Code>
                    }.into_view(),
                    SolutionState::Requested => view! {
                        <Code>"Fetching solution..."</Code>
                    }.into_view(),
                    SolutionState::NotFetched => view! {
                        <Code>"Please enter a scramble"</Code>
                    }.into_view()
                }
            }
            <div class:hidden=move || is_done_data.get()>
                <ProgressBar progress=create_signal(None).0 />
            </div>
            <h2>Exclude Solutions</h2>
            {move||{
                view!{
                    <Tabs mount=Mount::Once>
                        <Tab name="eo" label=view! {<span class=move||{if eo_step.get().is_some() { "" } else { "no-solution" }}>"EO"</span>}.into_view()>
                            <SolutionExcludeTab step_data=eo_step get_excluded=eo.excluded.0 set_excluded=eo.excluded.1 />
                        </Tab>
                        <Tab name="dr" label=view! {<span class=move||{if dr_step.get().is_some() { "" } else { "no-solution" }}>"DR"</span>}.into_view()>
                            <SolutionExcludeTab step_data=dr_step get_excluded=dr.excluded.0 set_excluded=dr.excluded.1/>
                        </Tab>
                        <Tab name="htr" label=view! {<span class=move||{if htr_step.get().is_some() { "" } else { "no-solution" }}>"HTR"</span>}.into_view()>
                            <SolutionExcludeTab step_data=htr_step get_excluded=htr.excluded.0 set_excluded=htr.excluded.1/>
                        </Tab>
                        <Tab name="fr" label=view! {<span class=move||{if fr_step.get().is_some() { "" } else { "no-solution" }}>"FR"</span>}.into_view()>
                            <SolutionExcludeTab step_data=fr_step get_excluded=fr.excluded.0 set_excluded=fr.excluded.1/>
                        </Tab>
                        <Tab name="fin" label=view! {<span class=move||{if fin_step.get().is_some() { "" } else { "no-solution" }}>"Finish"</span>}.into_view()>
                            <SolutionExcludeTab step_data=fin_step get_excluded=fin.excluded.0 set_excluded=fin.excluded.1/>
                        </Tab>
                    </Tabs>
                }.into_view()
            }}
        }
    }

    #[component]
    fn SolutionExcludeTab(step_data: Signal<Option<(SolutionStep, Algorithm)>>, get_excluded: Signal<HashSet<Algorithm>>, set_excluded: leptos::WriteSignal<HashSet<Algorithm>>) -> impl IntoView {
        view! {
            {move||if let Some((ss, alg)) = step_data.get() {
                view! {
                    <Button on_click=move|_|{
                        set_excluded.update(|list|{
                            list.insert(alg.clone().canonicalize());
                        });
                    }>{format!("Exclude: {}", ss.alg)}</Button>
                    // <label>{format!("Current solution: {}", ss.alg)}</label>
                }.into_view()
            } else {
                view! {
                    <label>The current solution does not contain this step</label>
                }.into_view()
            }}
            <h2>Excluded solutions</h2>
            <ul>
                {move||get_excluded.get().into_iter()
                    .map(|n| {
                        let n_1 = n.clone();
                        view! {
                        <li>
                            <button on:click=move|_|set_excluded.update(|set|{set.remove(&n_1);})
                                class="icon-button"
                                style:float="left"
                                style:font-size="15px">
                                <Icon icon=IoIcon::IoTrashOutline/>
                            </button>{n.to_string()}
                        </li>
                    }})
                    .collect_view()}
            </ul>
        }
    }

    fn find_step(sd: Signal<SolutionState>, kind: StepKind) -> Signal<Option<(SolutionStep, Algorithm)>> {
        Signal::derive(move||{
            if let SolutionState::Found(Ok(sol)) = sd.get() {
                let step_idx = sol.steps.iter().enumerate().find(|(_, x)|StepKind::from(x.variant) == kind).map(|(x, _)|x);
                if let Some(step_idx) = step_idx {
                    let mut full_alg = sol.steps.iter().take(step_idx + 1)
                        .map(|s|s.alg.clone())
                        .fold(Algorithm::new(), |mut acc, s|{
                            acc = acc + s;
                            acc
                        });
                    if kind == StepKind::FIN {
                        full_alg = full_alg.to_uninverted();
                    }
                    let step = sol.steps[step_idx].clone();
                    Some((step, full_alg.canonicalize()))
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    fn fetch_solution(request: SolverRequest, id: usize, solution_callback: RwSignal<SolutionState>, done_callback: RwSignal<bool>, cur_id: RwSignal<usize>) {
        let current_bytes = RefCell::<Vec<u8>>::new(vec![]);

        let body = serde_json::to_vec(&request).unwrap();
        let mut req = Request::post("https://joba.me/cubeapi/solve_stream?backend=multi_path_channel", body);
        // let mut req = Request::post("http://localhost:8049/solve_stream?backend=multi_path_channel", body);
        req.headers.insert("content-type".to_string(), "application/json".to_string());

        ehttp::streaming::fetch(req, move |res: ehttp::Result<ehttp::streaming::Part>| {
            let part = match res {
                Ok(part) => part,
                Err(err) => {
                    if cur_id.get_untracked() == id {
                        solution_callback.set(SolutionState::Found(Err(err)));
                        done_callback.set(true)
                    }
                    return std::ops::ControlFlow::Break(());
                }
            };

            match part {
                ehttp::streaming::Part::Response(response) => {
                    if response.ok {
                        std::ops::ControlFlow::Continue(())
                    } else {
                        std::ops::ControlFlow::Break(())
                    }
                }
                ehttp::streaming::Part::Chunk(chunk) => {
                    if cur_id.get_untracked() != id {
                        return std::ops::ControlFlow::Break(());
                    }
                    let mut start = 0;
                    for n in 0..(chunk.len()) {
                        if chunk[n] == b'\n' {
                            let mut val = current_bytes.take();
                            val.extend_from_slice(&chunk[start..n]);
                            match serde_json::from_slice::<SolverResponse>(val.as_slice()) {
                                Ok(res) => {
                                    if let Some(sol) = res.solution {
                                        solution_callback.set(SolutionState::Found(Ok(sol)))
                                    }
                                    if res.done {
                                        done_callback.set(true);
                                    }
                                },
                                Err(err) => {
                                    solution_callback.set(SolutionState::Found(Err(err.to_string())));
                                    done_callback.set(true);
                                },
                            }
                            start = n + 1; //Skip the newline
                        }
                    }
                    current_bytes.borrow_mut().extend_from_slice(&chunk[start..]);
                    std::ops::ControlFlow::Continue(())
                }
            }
        });
    }
}

#[derive(Clone, PartialEq, Eq)]
struct SolverRequestData {
    step_configs: Vec<StepConfig>,
    excluded: HashMap<StepKind, HashSet<Algorithm>>,
}

fn get_step_configs(eo: EOConfig, rzp: RZPConfig, dr: DRConfig, htr: HTRConfig, fr: FRConfig, fin: FinishConfig, settings: &SettingsState) -> Vec<StepConfig> {
    let relative = settings.is_relative();
    let advanced = settings.is_advanced();
    let default_variants = Some(vec!["ud".to_string(), "fb".to_string(), "lr".to_string()]);
    let mut steps_config = vec![];
    steps_config.push(StepConfig {
        kind: StepKind::EO,
        substeps: if advanced { Some(variants_to_string(eo.variants.0.get())) } else { default_variants.clone() },
        min: Some(eo.min_abs.0.get()),
        max: Some(eo.max_abs.0.get()),
        absolute_min: None,
        absolute_max: None,
        step_limit: None,
        quality: 10000,
        niss: Some(eo.niss.0.get()),
        params: Default::default(),
        excluded: eo.excluded.0.get(),
    });
    if dr.enabled.0.get() {
        let mut params = HashMap::new();
        if !dr.subsets.0.get().is_empty() {
            params.insert("subsets".to_string(), dr.subsets.0.get().join(","));
        }
        if dr.triggers.0.get().len() > 0 && dr.enforce_triggers.0.get() {
            steps_config.push(StepConfig {
                kind: StepKind::RZP,
                substeps: None,
                min: if !relative { Some(0) } else { Some(rzp.min_rel.0.get()) },
                max: if !relative { Some(3) } else { Some(rzp.max_rel.0.get()) },
                absolute_min: Some(rzp.min_abs.0.get()).filter(|_| !relative),
                absolute_max: Some(rzp.max_abs.0.get()).filter(|_| !relative),
                step_limit: None,
                quality: 10000,
                niss: Some(rzp.niss.0.get()),
                params: Default::default(),
                excluded: HashSet::new(),
            });
            params.insert("triggers".to_string(), dr.triggers.0.get().join(","));
            steps_config.push(StepConfig {
                kind: StepKind::DR,
                substeps: if advanced { Some(variants_to_string(dr.variants.0.get())) } else { default_variants.clone() },
                min: Some(dr.min_rel.0.get()).filter(|_|relative),
                max: Some(dr.max_rel.0.get()).filter(|_|relative),
                absolute_min: Some(dr.min_abs.0.get()).filter(|_| !relative),
                absolute_max: Some(dr.max_abs.0.get()).filter(|_| !relative),
                step_limit: None,
                quality: 10000,
                niss: Some(dr.niss.0.get()),
                params,
                excluded: dr.excluded.0.get(),
            });
        } else {
            steps_config.push(StepConfig {
                kind: StepKind::DR,
                substeps: if advanced { Some(variants_to_string(dr.variants.0.get())) } else { default_variants.clone() },
                min: Some(dr.min_rel.0.get()).filter(|_|relative),
                max: Some(dr.max_rel.0.get()).filter(|_|relative),
                absolute_min: Some(dr.min_abs.0.get()).filter(|_| !relative),
                absolute_max: Some(dr.max_abs.0.get()).filter(|_| !relative),
                step_limit: None,
                quality: 10000,
                niss: Some(dr.niss.0.get()),
                params,
                excluded: dr.excluded.0.get(),
            });
        }
    }
    if htr.enabled.0.get() {
        steps_config.push(StepConfig {
            kind: StepKind::HTR,
            substeps: if advanced { Some(variants_to_string(htr.variants.0.get())) } else { default_variants.clone() },
            min: Some(htr.min_rel.0.get()).filter(|_|relative),
            max: Some(htr.max_rel.0.get()).filter(|_|relative),
            absolute_min: Some(htr.min_abs.0.get()).filter(|_| !relative),
            absolute_max: Some(htr.max_abs.0.get()).filter(|_| !relative),
            step_limit: None,
            quality: 10000,
            niss: Some(htr.niss.0.get()),
            params: Default::default(),
            excluded: htr.excluded.0.get(),
        });
    }
    if fr.enabled.0.get() {
        steps_config.push(StepConfig {
            kind: if fin.leave_slice.0.get() {
                StepKind::FRLS
            } else {
                StepKind::FR
            },
            substeps: if advanced { Some(variants_to_string(fr.variants.0.get())) } else { default_variants.clone() },
            min: Some(fr.min_rel.0.get()).filter(|_|relative),
            max: Some(fr.max_rel.0.get()).filter(|_|relative),
            absolute_min: Some(fr.min_abs.0.get()).filter(|_| !relative),
            absolute_max: Some(fr.max_abs.0.get()).filter(|_| !relative),
            step_limit: None,
            quality: 10000,
            niss: Some(fr.niss.0.get()),
            params: Default::default(),
            excluded: fr.excluded.0.get(),
        });
    }
    if fin.enabled.0.get() {
        let mut params = HashMap::default();
        if htr.enabled.0.get() && !fr.enabled.0.get() {
            params.insert("htr-breaking".to_string(), fin.htr_breaking.0.get().to_string());
        }
        steps_config.push(StepConfig {
            kind: if fin.leave_slice.0.get() {
                StepKind::FINLS
            } else {
                StepKind::FIN
            },
            substeps: Some(vec!["ud".to_string(), "fb".to_string(), "lr".to_string()]),
            min: Some(fin.min_rel.0.get()).filter(|_|relative),
            max: Some(fin.max_rel.0.get()).filter(|_|relative),
            absolute_min: Some(fin.min_abs.0.get()).filter(|_| !relative),
            absolute_max: Some(fin.max_abs.0.get()).filter(|_| !relative),
            step_limit: None,
            quality: 10000,
            niss: Some(NissSwitchType::Never),
            params,
            excluded: fin.excluded.0.get(),
        });

    }
    steps_config
}

fn variants_to_string(variants: Vec<CubeAxis>) -> Vec<String> {
    variants.into_iter()
        .map(|a| Into::<SelectableAxis>::into(a).to_string())
        .collect()
}