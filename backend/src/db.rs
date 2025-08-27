use std::time::SystemTime;

use base64::Engine;
use cubelib::algs::Algorithm;
use cubelib::cube::turn::{ApplyAlgorithm, InvertibleMut};
use cubelib::cube::Cube333;
use cubelib::steps::step::StepConfig;
use cubelib::steps::tables::PruningTables333;

pub type Connection = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;

pub fn record_request(
    conn: &Connection,
    scramble: &Algorithm,
    step_configs: &Vec<StepConfig>,
    _: &PruningTables333,
) -> rusqlite::Result<()> {
    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("System time before unix epoch")
        .as_secs();
    let canonical = get_canonical_scramble_id(scramble);
    let encoded_steps = base64::engine::general_purpose::STANDARD
        .encode(serde_json::to_string(step_configs).unwrap());
    conn.execute(
        "INSERT INTO requests (id, timestamp, scramble, canonical_scramble_id, settings) VALUES (NULL, ?, ?, ?, ?)",
        (
            time,
            scramble.to_string(),
            canonical,
            encoded_steps
        )
    ).map(|_|())
}

fn get_canonical_scramble_id(scramble: &Algorithm) -> String {
    let mut cube = Cube333::default();
    cube.apply_alg(scramble);
    let normal_encoded = serialize_cube_to_base64(&cube);
    cube.invert();
    let inverse_encoded = serialize_cube_to_base64(&cube);
    std::cmp::min(normal_encoded, inverse_encoded)
}

pub fn init_db(conn: &Connection) -> rusqlite::Result<usize> {
    create_requests_table(&conn)
}

fn create_requests_table(conn: &Connection) -> rusqlite::Result<usize> {
    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS requests (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            scramble TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            canonical_scramble_id TEXT NOT NULL,
            settings TEXT NOT NULL
        )
    ",
        [],
    )?;
    conn.execute("CREATE INDEX IF NOT EXISTS id ON requests (scramble)", [])?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS id ON requests (canonical_scramble_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS id ON requests (canonical_scramble_inv_id)",
        [],
    )
}

fn serialize_cube_to_base64(cube: &Cube333) -> String {
    let mut cube_longs = cube.edges.get_edges_raw().to_vec();
    cube_longs.push(cube.corners.get_corners_raw());
    let bytes: Vec<u8> = cube_longs
        .into_iter()
        .flat_map(|x| x.to_le_bytes().into_iter())
        .collect();
    base64::engine::general_purpose::STANDARD.encode(&bytes)
}
