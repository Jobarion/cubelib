use std::collections::HashMap;
use std::str::FromStr;

use cubelib::algs::Algorithm;
use cubelib::defs::{NissSwitchType, StepKind};
use cubelib::solver_new::ar::ARBuilder;
use cubelib::solver_new::dr::{DRBuilder, RZPBuilder, RZPStep};
use cubelib::solver_new::eo::EOBuilder;
use cubelib::solver_new::finish::{DRFinishBuilder, FRFinishBuilder, HTRFinishBuilder};
use cubelib::solver_new::fr::FRBuilder;
use cubelib::solver_new::group::{StepGroup, StepPredicate};
use cubelib::solver_new::htr::HTRBuilder;
use cubelib::solver_new::util_cube::CubeState;
use cubelib::solver_new::util_steps::{FilterFirstN, FilterFirstNStepVariant};
use cubelib::steps::step::StepConfig;
use log::debug;
use pest::iterators::Pair;
use pest::Parser;
use crate::config::StepOverride;

#[derive(pest_derive::Parser)]
#[grammar = "steps.pest"]
struct StepsParser;

pub(crate) fn parse_steps<S: AsRef<str>>(s: S, prototypes: &HashMap<String, StepOverride>, cube_state: CubeState) -> Result<(StepGroup, StepKind), String> {
    let main = StepsParser::parse(Rule::main, s.as_ref()).unwrap().next().unwrap();

    let (group, target) = generate(main, None, prototypes, cube_state)?;
    if let Some(group) = group {
        Ok((group, target.kind))
    } else {
        Err("Unknown error".to_string())
    }
}

fn generate(p: Pair<Rule>, mut previous: Option<StepConfig>, prototypes: &HashMap<String, StepOverride>, cube_state: CubeState) -> Result<(Option<StepGroup>, StepConfig), String> {
    Ok(match p.as_rule() {
        Rule::step => parse_step(p, previous, prototypes, cube_state)?,
        Rule::sequence => {
            let mut steps = vec![];
            for inner in p.into_inner() {
                let (group, p_conf) = generate(inner, previous, prototypes, cube_state.clone())?;
                previous = Some(p_conf);
                if let Some(group) = group {
                    steps.push(group);
                }
            }
            if steps.is_empty() {
                (None, previous.unwrap())
            } else {
                (Some(StepGroup::sequential(steps)), previous.unwrap())
            }
        },
        Rule::parallel => {
            let mut steps = vec![];
            let mut target: Option<StepConfig> = None;
            for inner in p.into_inner() {
                let (group, kind) = generate(inner, previous.clone(), prototypes, cube_state.clone())?;
                if let Some(target) = target {
                    assert_eq!(target.kind, kind.kind);
                }
                target = Some(kind);
                if let Some(group) = group {
                    steps.push(group);
                }
            }
            if steps.is_empty() {
                (None, target.unwrap())
            } else {
                (Some(StepGroup::parallel(steps)), target.unwrap())
            }
        }
        p => {
            println!("{p:?}");
            unreachable!()
        },
    })
}

fn parse_step(p: Pair<Rule>, previous: Option<StepConfig>, prototypes: &HashMap<String, StepOverride>, cube_state: CubeState) -> Result<(Option<StepGroup>, StepConfig), String> {
    let mut inner = p.into_inner();
    let kind = inner.next().unwrap();
    assert_eq!(Rule::kind, kind.as_rule());
    let kind = kind.as_str();
    let prototype = prototypes.get(kind);
    let kind = if let Some(prototype) = prototype {
        prototype.kind.clone()
    } else {
        StepKind::from_str(kind).unwrap()
    };
    let mut variants = vec![];
    let mut step_prototype = StepConfig {
        kind: kind.clone(),
        substeps: None,
        min: None,
        max: None,
        absolute_min: None,
        absolute_max: None,
        niss: None,
        step_limit: None,
        quality: 0,
        params: HashMap::new()
    };
    if let Some(prototype) = prototype {
        for (key, value) in &prototype.parameters {
            parse_kv(&mut step_prototype, key, value)?;
        }
    }
    loop {
        let next = if let Some(next) = inner.next() {
            next
        } else {
            break
        };
        match next.as_rule() {
            Rule::variant => variants.push(next.as_str().to_string()),
            Rule::key => {
                let key = next.as_str();
                let value = inner.next().unwrap().as_str();
                parse_kv(&mut step_prototype, key, value)?;
            },
            x => {
                println!("{x:?}")
            }
            // _ => unreachable!()
        }
    }
    if !variants.is_empty() {
        step_prototype.substeps = Some(variants);
    }
    let mut step_prototype_c = step_prototype.clone();
    let limit = step_prototype.params.remove("step-limit");
    let max_use = step_prototype.params.remove("max-use");

    let mut previous_kind = previous.as_ref().map(|s|s.kind.clone());
    debug!("{:?} -> {} (current state is {:?})", previous_kind, kind, cube_state);
    if previous_kind.is_none() {
        previous_kind = match cube_state {
            CubeState::EO(_) => Some(StepKind::EO),
            CubeState::DR(_) => Some(StepKind::DR),
            CubeState::TripleDR => Some(StepKind::DR),
            CubeState::HTR => Some(StepKind::HTR),
            CubeState::FR(_) => Some(StepKind::FR),
            CubeState::Solved => Some(StepKind::FIN),
            _ => None,
        };
        if let Some(k) = previous_kind.as_ref() {
            debug!("Replacing previous state with {}", k);
        }
    }
    let mut step = match (previous_kind, kind) {
        (None, StepKind::EO) => Some(EOBuilder::try_from(step_prototype).map_err(|_|"Failed to parse EO step")?.build()),
        (Some(StepKind::EO), StepKind::RZP) => None,
        (Some(StepKind::RZP), StepKind::DR) => {
            let triggers = step_prototype.params.remove("triggers").ok_or("Found RZP, but DR step has no triggers".to_string())?;
            let rzp_builder = RZPBuilder::try_from(previous.unwrap()).map_err(|_|"Failed to parse RZP step")?;
            Some(DRBuilder::try_from(step_prototype).map_err(|_|"Failed to parse DR step")?
                .triggers(triggers.split(",")
                    .map(Algorithm::from_str)
                    .collect::<Result<_, _>>()
                    .map_err(|_|"Unable to parse algorithm")?)
                .rzp(rzp_builder)
                .build())
        },
        (Some(StepKind::EO), StepKind::AR) => Some(ARBuilder::try_from(step_prototype).map_err(|_|"Failed to parse ARM step")?.build()),
        (Some(StepKind::AR), StepKind::DR) => {
            Some(DRBuilder::try_from(step_prototype).map_err(|_|"Failed to parse DR step")?
                .from_ar()
                .build())
        },
        (Some(StepKind::EO), StepKind::DR) => {
            Some(match step_prototype.params.remove("triggers") {
                None => DRBuilder::try_from(step_prototype).map_err(|_|"Failed to parse DR step")?.build(),
                Some(triggers) => {
                    let rzp = RZPStep::builder()
                        .max_length(step_prototype.max.unwrap_or(3).min(3) as usize)
                        .max_absolute_length(step_prototype.absolute_max.unwrap_or(6).min(6) as usize);
                    DRBuilder::try_from(step_prototype).map_err(|_|"Failed to parse DR step")?
                        .triggers(triggers.split(",")
                            .map(Algorithm::from_str)
                            .collect::<Result<_, _>>()
                            .map_err(|_|"Unable to parse algorithm")?)
                        .rzp(rzp)
                        .build()
                }
            })
        },
        (Some(StepKind::DR), StepKind::HTR) => Some(HTRBuilder::try_from(step_prototype).map_err(|_|"Failed to parse HTR step")?.build()),
        (Some(StepKind::HTR), StepKind::FR) | (Some(StepKind::HTR), StepKind::FRLS)  => Some(FRBuilder::try_from(step_prototype).map_err(|_|"Failed to parse FR step")?.build()),
        (Some(StepKind::DR), StepKind::FIN) => Some(DRFinishBuilder::try_from(step_prototype).map_err(|_|"Failed to parse FIN step")?.build()),
        (Some(StepKind::FR), StepKind::FIN) => Some(FRFinishBuilder::try_from(step_prototype).map_err(|_|"Failed to parse FIN step")?.build()),
        (Some(StepKind::FRLS), StepKind::FINLS) => Some(FRFinishBuilder::try_from(step_prototype).map_err(|_|"Failed to parse FIN step")?.build()),
        (Some(StepKind::HTR), StepKind::FIN) | (Some(StepKind::HTR), StepKind::FINLS) => Some(HTRFinishBuilder::try_from(step_prototype).map_err(|_|"Failed to parse FIR step")?.build()),
        (None, x) => return Err(format!("{x:?} is not supported as a first step", )),
        (Some(a), b) => return Err(format!("Step order {a:?} > {b:?} is not supported")),
    };

    if let Some(step) = step.as_mut() {
        if let Some(max_use) = max_use {
            let filters: Result<Vec<Box<dyn StepPredicate>>, String> = max_use.split(",")
                .flat_map(|x|{
                    x.split_once(":")
                        .map(|(a, b)|{
                            let kind = StepKind::from_str(a).unwrap();
                            let n = usize::from_str(b).map_err(|_|"Failed to parse max use limit".to_string());
                            n.map(|n|FilterFirstNStepVariant::new(kind, n))
                        })
                })
                .collect();
            step.with_predicates(filters?);
        }
        if let Some(limit) = limit {
            step.with_predicates(vec![FilterFirstN::new(usize::from_str(limit.as_str()).map_err(|_|"Failed to parse step limit")?)]);
        }
    } else {
        if let Some(limit) = limit {
            step_prototype_c.params.insert("step-limit".to_string(), limit);
        }
        if let Some(max_use) = max_use {
            step_prototype_c.params.insert("max-use".to_string(), max_use);
        }
    }

    Ok((step, step_prototype_c))
}

fn parse_kv(step_prototype: &mut StepConfig, key: &str, value: &str) -> Result<(), String> {
    match key {
        "limit" => step_prototype.step_limit = Some(usize::from_str(value).map_err(|x| format!("Unable to parse value '{value}' for count. '{x}'"))?),
        key @ "min" | key @ "min-rel" => step_prototype.min = Some(u8::from_str(value).map_err(|x| format!("Unable to parse value '{value}' for {key}. '{x}'"))?),
        key @ "max" | key @ "max-rel" => step_prototype.max = Some(u8::from_str(value).map_err(|x| format!("Unable to parse value '{value}' for {key}. '{x}'"))?),
        "min-abs" => step_prototype.absolute_min = Some(u8::from_str(value).map_err(|x| format!("Unable to parse value '{value}' for min-abs. '{x}'"))?),
        "max-abs" => step_prototype.absolute_max = Some(u8::from_str(value).map_err(|x| format!("Unable to parse value '{value}' for max-abs. '{x}'"))?),
        "niss" => step_prototype.niss = Some(match value {
            "always" | "true" => NissSwitchType::Always,
            "before" => NissSwitchType::Before,
            "none" | "never" | "false" => NissSwitchType::Never,
            x => Err(format!("Invalid NISS type {x}. Expected one of 'always', 'before', 'none'"))?
        }),
        _ => {
            step_prototype.params.insert(key.to_string(), value.to_string());
        },
    }
    Ok(())
}