use std::collections::HashMap;
use std::str::FromStr;
use actix_web::web::to;
use cubelib::algs::Algorithm;

use cubelib::defs::StepKind;
use cubelib::solution::{Solution, SolutionStep};
use cubelib_interface::StepConfig;
use log::warn;
use rusqlite::{params, params_from_iter, ToSql};
use rusqlite::types::Value;

pub type Connection = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;

pub fn insert_solution(mut conn: &mut Connection, scramble: Algorithm, solution: Option<Solution>, step_configs: Vec<StepConfig>, quality: usize) -> rusqlite::Result<()> {
    let last_step = step_configs.last().map_or(-1, |sk|sk.kind as usize as isize);
    let tx = conn.transaction()?;
    tx.execute("INSERT INTO solutions (id, scramble, quality, last_step, solved) VALUES (NULL, ?, ?, ?, ?)", (scramble.to_string(), quality, last_step, solution.is_some() as usize))?;
    let id = tx.last_insert_rowid();

    let mut map: HashMap<StepKind, (StepConfig, Option<SolutionStep>)> = HashMap::default();
    for step_config in step_configs {
        let sl = solution.iter()
            .flat_map(|s|s.steps.iter())
            .find(|s| s.kind == step_config.kind)
            .cloned();
        map.insert(step_config.kind, (step_config, sl));
    }

    for (config, solution) in map.values() {
        let props = format_props(config.params.clone());
        let variants = format_variants(config.substeps.clone());

        tx.execute(
            "INSERT INTO step_settings (solution_id, step_kind, min, max, niss, variants, props) VALUES (?, ?, ?, ?, ?, ?, ?)",
            (id, config.kind as usize, config.min, config.max, config.niss as usize, variants, props)
        )?;
        if let Some(sol) = solution {
            let normal = sol.alg.normal_moves.iter().map(|x|x.to_string()).collect::<Vec<String>>().join("");
            let inverse = sol.alg.inverse_moves.iter().map(|x|x.to_string()).collect::<Vec<String>>().join("");
            let variant = sol.variant.splitn(2, "-").collect::<Vec<&str>>()[0].to_string();
            tx.execute(
                "INSERT INTO step_solutions (solution_id, step_kind, length, variant, normal, inverse) VALUES (?, ?, ?, ?, ?, ?)",
                (id, config.kind as usize, sol.alg.len(), variant, normal, inverse)
            )?;
        }
    }
    tx.commit()
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

pub fn load_solution(conn: &Connection, scramble: &Algorithm, step_configs: Vec<StepConfig>, quality: usize) -> rusqlite::Result<Option<Solution>> {
    let where_part = "(sset.step_kind = ? AND sset.min <= ? AND sset.max >= ? AND ssol.length >= ? AND ssol.length <= ? AND sset.niss = ? AND instr(sset.variants, ?) > 0 AND instr(?, ssol.variant) > 0 AND sset.props = ?)";

    let where_part = vec![where_part].into_iter()
        .cycle()
        .take(step_configs.len())
        .collect::<Vec<&str>>()
        .join(" OR ");

    let query = format!("
        SELECT ssol.solution_id, ssol.step_kind, ssol.variant, ssol.normal, ssol.inverse, s.solved FROM step_solutions ssol
            INNER JOIN step_settings sset ON ssol.solution_id = sset.solution_id AND ssol.step_kind = sset.step_kind
            INNER JOIN solutions s ON s.id = ssol.solution_id
            WHERE s.scramble = ? AND s.quality >= ? AND s.last_step = ? AND ({where_part})
    ");


    let first_params = vec![Value::from(scramble.to_string()), Value::from(quality as u32), Value::from(step_configs.last().map_or(-1, |sk|sk.kind as usize as isize))];

    let params = step_configs.iter()
        .flat_map(|sc| {
            let props = format_props(sc.params.clone());
            let variants = format_variants(sc.substeps.clone());

            vec![
                Value::from(sc.kind as usize as u32),
                Value::from(sc.min),
                Value::from(sc.max),
                Value::from(sc.min),
                Value::from(sc.max),
                Value::from(sc.niss as usize as u32),
                Value::from(variants.clone()),
                Value::from(variants),
                Value::from(props),
            ].into_iter()
        });

    let params = first_params.into_iter().chain(params);
    let params = params_from_iter(params);

    let mut result = conn.prepare(query.as_str())?;

    let parts: Vec<(usize, SolutionStep)> = result.query_map(params, |row| {
        let sol_id: usize = row.get(0)?;
        let kind: StepKind = match row.get(1)? {
            0 => StepKind::EO,
            1 => StepKind::RZP,
            2 => StepKind::DR,
            3 => StepKind::HTR,
            4 => StepKind::FR,
            5 => StepKind::FRLS,
            6 => StepKind::FIN,
            _ => unreachable!()
        };
        let variant: String = row.get(2)?;
        let normal: String = row.get(3)?;
        let inverse: String = row.get(4)?;
        let alg = Algorithm::from_str(format!("{normal} ({inverse})").as_str()).unwrap();
        Ok((sol_id, SolutionStep {
            kind,
            variant,
            alg,
        }))
    }).and_then(Iterator::collect)?;

    let mut solutions: HashMap<usize, Vec<SolutionStep>> = HashMap::new();
    for (id, step) in parts {
        if !solutions.contains_key(&id) {
            solutions.insert(id, vec![]);
        }
        solutions.get_mut(&id).unwrap().push(step);
    }
    let solutions: Vec<Solution> = solutions.into_values()
        .filter(|steps|steps.len() == step_configs.len())
        .map(|mut steps| {
            steps.sort_by(|a, b|(a.kind as usize).cmp(&(b.kind as usize)));
            Solution {
                steps,
                ends_on_normal: true, //Don't care right now
            }
        })
        .collect();

    if solutions.len() > 1 {
        warn!("Found multiple matching solutions. This shouldn't happen");
    }

    Ok(solutions.first().cloned())
}

pub fn init_db(conn: &Connection) -> rusqlite::Result<usize> {
    create_solutions_table(&conn)?;
    create_step_settings_table(&conn)?;
    create_step_solution_table(&conn)
}

fn create_solutions_table(conn: &Connection) -> rusqlite::Result<usize> {
    conn.execute("
        CREATE TABLE IF NOT EXISTS solutions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            scramble TEXT NOT NULL,
            quality INTEGER NOT NULL,
            last_step INTEGER NOT NULL,
            solved INTEGER NOT NULL
        )
    ", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS id ON solutions (id)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS scramble ON solutions (scramble)", [])
}

fn create_step_settings_table(conn: &Connection) -> rusqlite::Result<usize> {
    conn.execute("
        CREATE TABLE IF NOT EXISTS step_settings (
            solution_id INTEGER NOT NULL,
            step_kind INTEGER NOT NULL,
            min INTEGER NOT NULL,
            max INTEGER NOT NULL,
            niss INTEGER NOT NULL,
            variants TEXT NOT NULL,
            props TEXT NOT NULL,
            FOREIGN KEY(solution_id) references solutions(id)
        )
    ", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS id ON step_settings (solution_id)", [])
}

fn create_step_solution_table(conn: &Connection) -> rusqlite::Result<usize> {
    conn.execute("
        CREATE TABLE IF NOT EXISTS step_solutions (
            solution_id INTEGER NOT NULL,
            step_kind INTEGER NOT NULL,
            length INTEGER NOT NULL,
            variant TEXT NOT NULL,
            normal TEXT NOT NULL,
            inverse TEXT NOT NULL,
            FOREIGN KEY(solution_id) references solutions(id)
        )
    ", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS id ON step_settings (solution_id)", [])
}