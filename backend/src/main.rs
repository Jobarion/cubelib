use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use cubelib::puzzles::c333::steps::tables::PruningTables333;
use log::LevelFilter;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use simple_logger::SimpleLogger;

mod controller;
mod db;

struct AppData {
    pruning_tables: Arc<PruningTables333>,
    pool: Pool<SqliteConnectionManager>
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

    let pruning_tables = Arc::new(pruning_tables);

    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .app_data(web::Data::new(AppData {
                pruning_tables: pruning_tables.clone(),
                pool: pool.clone()
            }))
            .wrap(cors)
            .service(controller::solve_stream)
    })
        .bind(("127.0.0.1", 8049))?
        .run()
        .await
}