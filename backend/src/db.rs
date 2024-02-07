use std::collections::HashMap;
use std::str::FromStr;

use cubelib::algs::Algorithm;
use cubelib::defs::{NissSwitchType, StepKind};
use cubelib::puzzles::c333::Turn333;
use cubelib::solver::solution::{Solution, SolutionStep};
use cubelib::steps::step::StepConfig;
use log::warn;
use rusqlite::params_from_iter;
use rusqlite::types::Value;

pub type Connection = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;

pub fn insert_solution(mut conn: &mut Connection, scramble: Algorithm<Turn333>, solution: Option<Solution<Turn333>>, step_configs: Vec<StepConfig>, quality: usize) -> rusqlite::Result<()> {
    let last_step = step_configs.last().map_or(-1, |sk|kind_to_id(sk.kind.clone()));
    let tx = conn.transaction()?;
    tx.execute("INSERT INTO solutions (id, scramble, quality, last_step, solved) VALUES (NULL, ?, ?, ?, ?)", (scramble.to_string(), quality, last_step, solution.is_some() as usize))?;
    let id = tx.last_insert_rowid();

    let mut map: HashMap<StepKind, (StepConfig, Option<SolutionStep<Turn333>>)> = HashMap::default();
    for step_config in step_configs {
        let sl = solution.iter()
            .flat_map(|s|s.steps.iter())
            .find(|s| s.kind == step_config.kind)
            .cloned();
        map.insert(step_config.kind.clone(), (step_config, sl));
    }

    for (config, solution) in map.values() {
        let props = format_props(config.params.clone());
        let variants = format_variants(config.substeps.clone().unwrap_or(vec![]).clone());

        tx.execute(
            "INSERT INTO step_settings (solution_id, step_kind, min, max, niss, variants, props) VALUES (?, ?, ?, ?, ?, ?, ?)",
            (id, kind_to_id(config.kind.clone()) as usize, config.min, config.max, config.niss.unwrap_or(NissSwitchType::Never) as usize, variants, props)
        )?;
        if let Some(sol) = solution {
            let normal = sol.alg.normal_moves.iter().map(|x|x.to_string()).collect::<Vec<String>>().join("");
            let inverse = sol.alg.inverse_moves.iter().map(|x|x.to_string()).collect::<Vec<String>>().join("");
            let variant = sol.variant.splitn(2, "-").collect::<Vec<&str>>()[0].to_string();
            tx.execute(
                "INSERT INTO step_solutions (solution_id, step_kind, length, variant, normal, inverse) VALUES (?, ?, ?, ?, ?, ?)",
                (id, kind_to_id(config.kind.clone()) as usize, sol.alg.len(), variant, normal, inverse)
            )?;
        }
    }
    tx.commit()
}

fn kind_to_id(kind: StepKind) -> isize {
    match kind {
        StepKind::EO => 0,
        StepKind::RZP => 1,
        StepKind::DR => 2,
        StepKind::HTR => 3,
        StepKind::FR => 4,
        StepKind::FRLS => 5,
        StepKind::FIN => 6,
        StepKind::Other(_) => 7,
    }
}

fn format_props(props: HashMap<String, String>) -> String {
    let mut props: Vec<(String, String)> = props.into_iter().collect();
    props.sort_by(|s1, s2|s1.0.cmp(&s2.0));
    props.iter()
        .map(|x|format!("{}={}", x.0, x.1))
        .collect::<Vec<String>>()
        .join(";")
}

fn format_variants(mut variants: Vec<String>) -> String {
    variants.sort();
    variants.join(";")
}