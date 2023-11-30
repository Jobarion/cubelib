use std::time::SystemTime;
use base64::Engine;

use cubelib::algs::Algorithm;
use cubelib::defs::{NissSwitchType, StepKind};
use cubelib::puzzles::c333::{Cube333, Turn333};
use cubelib::puzzles::c333::steps::solver::build_steps;
use cubelib::puzzles::c333::steps::tables::PruningTables333;
use cubelib::puzzles::puzzle::{ApplyAlgorithm, InvertibleMut};
use cubelib::solver::{CancellationToken, solve_steps};
use cubelib::steps::step::StepConfig;

pub type Connection = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;

pub fn record_request(conn: &Connection, scramble: &Algorithm<Turn333>, step_configs: &Vec<StepConfig>, tables: &PruningTables333) -> rusqlite::Result<()> {
    let time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("System time before unix epoch").as_secs();
    let canonical = get_canonical_scrambles(scramble, tables);
    let encoded_steps = base64::engine::general_purpose::STANDARD.encode(serde_json::to_string(step_configs).unwrap());
    conn.execute(
        "INSERT INTO requests (id, scramble, timestamp, canonical_scramble, canonical_scramble_inv, settings) VALUES (NULL, ?, ?, ?, ?, ?)",
        (
            scramble.to_string(),
            time,
            canonical.0.to_string(),
            canonical.1.to_string(),
            encoded_steps
        )
    ).map(|_|())
}

fn get_canonical_scrambles(scramble: &Algorithm<Turn333>, tables: &PruningTables333) -> (Algorithm<Turn333>, Algorithm<Turn333>) {
    let step_config = build_steps(vec![
            StepConfig::new(StepKind::EO),
            StepConfig::new(StepKind::DR),
            StepConfig::new(StepKind::HTR),
            StepConfig::new(StepKind::FR),
            StepConfig::new(StepKind::FIN),
    ].into_iter().map(|mut s|{
        s.quality = 1;
        s.niss = Some(NissSwitchType::Never);
        s.step_limit = Some(1);
        s
    }).collect(), tables).unwrap();
    let mut cube = Cube333::default();
    cube.apply_alg(scramble);
    let mut cube_inverse = cube.clone();
    cube_inverse.invert();
    let canonical_normal = solve_steps(cube, &step_config, CancellationToken::new()).next().expect("Expected solution");
    let canonical_inverse = solve_steps(cube_inverse, &step_config, CancellationToken::new()).next().expect("Expected solution");
    (canonical_normal.into(), canonical_inverse.into())
}

pub fn init_db(conn: &Connection) -> rusqlite::Result<usize> {
    create_requests_table(&conn)
}

fn create_requests_table(conn: &Connection) -> rusqlite::Result<usize> {
    conn.execute("
        CREATE TABLE IF NOT EXISTS requests (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            scramble TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            canonical_scramble TEXT NOT NULL,
            canonical_scramble_inv TEXT NOT NULL,
            settings TEXT NOT NULL
        )
    ", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS id ON requests (scramble)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS id ON requests (canonical_scramble)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS id ON requests (canonical_scramble_inv)", [])
}
