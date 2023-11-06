use std::str::FromStr;
use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{App, HttpServer, Responder, web};
use cubelib::algs::Algorithm;
use cubelib::cube::ApplyAlgorithm;
use cubelib::cube::NewSolved;
use cubelib::solution::{Solution, SolutionStep};
use cubelib::tables::PruningTables;
use cubelib_interface::SolverResponse;
use log::LevelFilter;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use simple_logger::SimpleLogger;
use db::Connection;

mod controller;
mod db;


struct AppData {
    pruning_tables: Arc<PruningTables>,
    pool: Pool<SqliteConnectionManager>
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .init()
        .unwrap();

    let manager = SqliteConnectionManager::file("solves.db");
    let pool = Pool::new(manager).unwrap();
    let conn = pool.get().expect("Connection required");
    db::init_db(&conn).unwrap();

    let mut pruning_tables = PruningTables::new();
    pruning_tables.gen_eo();
    pruning_tables.gen_dr();
    pruning_tables.gen_htr();
    pruning_tables.gen_fr();
    pruning_tables.gen_fr_leave_slice();
    pruning_tables.gen_fr_finish();

    let pruning_tables = Arc::new(pruning_tables);

    HttpServer::new(move || {
        let cors = Cors::permissive();
            // .allowed_origin("http://localhost:8080");
        App::new()
            .app_data(web::Data::new(AppData {
                pruning_tables: pruning_tables.clone(),
                pool: pool.clone()
            }))
            .wrap(cors)
            .service(controller::solve)
    })
        .bind(("127.0.0.1", 8049))?
        .run()
        .await
}