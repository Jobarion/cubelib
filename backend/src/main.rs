use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use cubelib::algs::Algorithm;
use cubelib::steps::tables::PruningTables333;
use log::LevelFilter;
use moka::sync::Cache;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use simple_logger::SimpleLogger;

mod controller;
mod db;

struct AppData {
    pruning_tables: Arc<PruningTables333>,
    pool: Pool<SqliteConnectionManager>,
    debounce_cache: Arc<Cache<u64, SocketAddr>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    let manager = SqliteConnectionManager::file("requests.db");
    let pool = Pool::new(manager).unwrap();
    let conn = pool.get().expect("Connection required");
    db::init_db(&conn).unwrap();

    let mut pruning_tables = PruningTables333::new();
    pruning_tables.gen_eo();
    pruning_tables.gen_dr();
    pruning_tables.gen_htr();
    pruning_tables.gen_fr();
    pruning_tables.gen_fr_leave_slice();
    pruning_tables.gen_fr_finish();
    pruning_tables.gen_htr_finish();
    pruning_tables.gen_htr_leave_slice_finish();

    let pruning_tables = Arc::new(pruning_tables);
    let debounce_cache = Arc::new(Cache::builder()
        .time_to_live(Duration::from_millis(500))
        .max_capacity(1000)
        .build()
    );

    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .app_data(web::Data::new(AppData {
                pruning_tables: pruning_tables.clone(),
                pool: pool.clone(),
                debounce_cache: debounce_cache.clone(),
            }))
            .wrap(cors)
            .service(controller::solve_stream)
    })
        .bind(("127.0.0.1", 8049))?
        .run()
        .await
}
