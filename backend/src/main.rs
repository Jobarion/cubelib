use std::collections::HashMap;
use actix_web::{error, get, post, web, App, HttpServer, Responder, Error, HttpResponse};
use cubelib::algs::{Algorithm, Solution};
use cubelib::cube::ApplyAlgorithm;
use cubelib::cubie::CubieCube;
use cubelib::cube::NewSolved;
use cubelib::df_search::NissSwitchType;
use cubelib::steps::step::{StepKind};
use cubelib::tables::PruningTables;
use std::str::FromStr;
use std::sync::Arc;
use actix_cors::Cors;
use cubelib_interface::*;
use log::LevelFilter;
use simple_logger::SimpleLogger;


#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .init()
        .unwrap();

    let mut pruning_tables = PruningTables::new();
    pruning_tables.gen_eo();
    pruning_tables.gen_dr();
    pruning_tables.gen_htr();
    pruning_tables.gen_fr();
    pruning_tables.gen_fr_leave_slice();
    pruning_tables.gen_fr_finish();

    let pruning_tables = web::Data::new(Arc::new(pruning_tables));

    HttpServer::new(move || {
        let cors = Cors::permissive();
            // .allowed_origin("http://localhost:8080");
        App::new()
            .app_data(pruning_tables.clone())
            .wrap(cors)
            .service(solve)
    })
        .bind(("127.0.0.1", 8081))?
        .run()
        .await
}

#[post("/solve")]
async fn solve(steps: web::Json<SolverRequest>, tables: web::Data<Arc<PruningTables>>) -> actix_web::Result<impl Responder> {
    let scramble = Algorithm::from_str(steps.scramble.as_str()).map_err(|_|error::ErrorBadRequest("Invalid algorithm"))?;

    let mut cube = CubieCube::new_solved();
    cube.apply_alg(&scramble);

    let steps: Vec<cubelib::steps::step::StepConfig> = steps.steps.iter()
        .map(|step| cubelib::steps::step::StepConfig {
            kind: map_step_kind(&step.kind),
            substeps: Some(step.substeps.clone()),
            min: Some(step.min),
            max: Some(step.max),
            step_limit: None,
            quality: steps.quality.unwrap_or(1000),
            niss: Some(map_niss_type(&step.niss)),
            params: step.params.clone(),
        }).collect();
    println!("{:?}", steps);
    let steps = cubelib_solver::solver::build_steps(steps, &tables).map_err(|err|error::ErrorBadRequest(err))?;
    let solutions = cubelib_solver::solver::solve_steps(cube, &steps);
    let mut solutions = cubelib::stream::distinct_algorithms(solutions);

    Ok(HttpResponse::Ok().json(solutions.next().map(SolverResponse::from)))
}

//Yes I know this is stupid
pub fn map_step_kind(kind: &cubelib_interface::StepKind) -> cubelib::steps::step::StepKind {
    match kind {
        cubelib_interface::StepKind::EO => StepKind::EO,
        cubelib_interface::StepKind::RZP => StepKind::RZP,
        cubelib_interface::StepKind::DR => StepKind::DR,
        cubelib_interface::StepKind::HTR => StepKind::HTR,
        cubelib_interface::StepKind::FR => StepKind::FR,
        cubelib_interface::StepKind::FRLS => StepKind::FRLS,
        cubelib_interface::StepKind::FIN => StepKind::FIN
    }
}


pub fn map_niss_type(kind: &cubelib_interface::NissSwitchType) -> cubelib::df_search::NissSwitchType {
    match kind {
        cubelib_interface::NissSwitchType::Always => NissSwitchType::Always,
        cubelib_interface::NissSwitchType::Before => NissSwitchType::Before,
        cubelib_interface::NissSwitchType::Never => NissSwitchType::Never,
    }
}
