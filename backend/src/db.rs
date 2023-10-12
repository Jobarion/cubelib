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
    let tx = conn.transaction()?;
    tx.execute("INSERT INTO solutions (id, scramble, quality, solved) VALUES (NULL, ?, ?, ?)", (scramble.to_string(), quality, solution.is_some() as usize))?;
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
        let length = solution.clone().map_or(0, |sol|sol.alg.len());
        let mut props: Vec<(String, String)> = config.params.clone().into_iter().collect();
        props.sort_by(|s1, s2|s1.0.cmp(&s2.0));
        let props = props.iter()
            .map(|x|format!("{}={}", x.0, x.1))
            .collect::<Vec<String>>()
            .join(";");
        let mut variants = config.substeps.clone();
        variants.sort();
        let variants = variants.join(";");

        tx.execute(
            "INSERT INTO step_settings (solution_id, step_kind, min, max, niss, variants, props) VALUES (?, ?, ?, ?, ?, ?, ?)",
            (id, config.kind as usize, length, config.max, config.niss as usize, variants, props)
        )?;
        if let Some(sol) = solution {
            let normal = sol.alg.normal_moves.iter().map(|x|x.to_string()).collect::<Vec<String>>().join("");
            let inverse = sol.alg.inverse_moves.iter().map(|x|x.to_string()).collect::<Vec<String>>().join("");
            tx.execute(
                "INSERT INTO step_solutions (solution_id, step_kind, variant, normal, inverse) VALUES (?, ?, ?, ?, ?)",
                (id, config.kind as usize, sol.variant.clone(), normal, inverse)
            )?;
        }
    }
    tx.commit()
}

//TODO very broken
pub fn load_solution(conn: &Connection, scramble: &Algorithm, step_configs: Vec<StepConfig>, quality: usize) -> rusqlite::Result<Option<Solution>> {
    let where_part = "(sset.step_kind = ? AND sset.min >= ? AND sset.max <= ? AND sset.niss = ? AND instr(sset.variants, ?) > 0 AND instr(?, ssol.variant) > 0 AND sset.props = ?)";

    let where_part = vec![where_part].into_iter()
        .cycle()
        .take(step_configs.len())
        .collect::<Vec<&str>>()
        .join(" OR ");

    let query = format!("
        SELECT ssol.solution_id, ssol.step_kind, ssol.variant, ssol.normal, ssol.inverse FROM solutions s
            INNER JOIN step_settings sset ON s.id = sset.solution_id
            INNER JOIN step_solutions ssol ON s.id = ssol.solution_id
            WHERE
                s.scramble = ? AND
                s.quality > ? AND
                {where_part}
    ");

    let params = step_configs.iter()
        .flat_map(|sc| {
            let mut props: Vec<(String, String)> = sc.params.clone().into_iter().collect();
            props.sort_by(|s1, s2|s1.0.cmp(&s2.0));
            let props = props.iter()
                .map(|x|format!("{}={}", x.0, x.1))
                .collect::<Vec<String>>()
                .join(";");
            let mut variants = sc.substeps.clone();
            variants.sort();
            let variants = variants.join(";");

            vec![
                Value::from(sc.kind as usize as u32),
                Value::from(sc.min),
                Value::from(sc.max),
                Value::from(sc.niss as usize as u32),
                Value::from(variants.clone()),
                Value::from(variants),
                Value::from(props),
            ].into_iter()
        });

    let params = vec![Value::from(scramble.to_string()), Value::from(quality as u32)].into_iter().chain(params);
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

    if solutions.len() > 1 {
        warn!("Found multiple matching solutions. This shouldn't happen");
    }

    Ok(solutions.into_values()
        .map(|mut steps| {
            steps.sort_by(|a, b|(a.kind as usize).cmp(&(b.kind as usize)));
            Solution {
                steps,
                ends_on_normal: true, //Don't care right now
            }
        })
        .next())
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
            scramble TEXT,
            quality INTEGER,
            solved INTEGER
        )
    ", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS id ON solutions (id)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS scramble ON solutions (scramble)", [])
}

fn create_step_settings_table(conn: &Connection) -> rusqlite::Result<usize> {
    conn.execute("
        CREATE TABLE IF NOT EXISTS step_settings (
            solution_id INTEGER,
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
            solution_id INTEGER,
            step_kind INTEGER NOT NULL,
            variant TEXT,
            normal TEXT,
            inverse TEXT,
            FOREIGN KEY(solution_id) references solutions(id)
        )
    ", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS id ON step_settings (solution_id)", [])
}