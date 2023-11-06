use std::str::FromStr;
use std::time::Duration;

use actix_web::{error, HttpResponse, post, Responder, web};
use cubelib::algs::Algorithm;
use cubelib::cube::{ApplyAlgorithm, NewSolved};
use cubelib::cubie::CubieCube;
use cubelib_interface::{SolverRequest, SolverResponse};
use log::{error, info};

use crate::{AppData, db};

#[post("/solve")]
pub async fn solve(steps: web::Json<SolverRequest>, app_data: web::Data<AppData>) -> actix_web::Result<impl Responder> {
    let scramble = Algorithm::from_str(steps.scramble.as_str()).map_err(|_|error::ErrorBadRequest("Invalid algorithm"))?;

    let quality = steps.quality.unwrap_or(1000);

    let conn = app_data.pool.get();
    if let Ok(conn) = conn {
        let cached = db::load_solution(&conn, &scramble, steps.steps.clone(), quality);
        if let Ok(Some(solution)) = cached {
            info!("Cache hit");
            return Ok(HttpResponse::Ok().json(SolverResponse::from(solution)));
        } else if let Err(err) = cached {
            error!("{err}");
        }
    } else if let Err(err) = conn {
        error!("{err}");
    }

    let steps_configs: Vec<cubelib::steps::step::StepConfig> = steps.steps.iter()
        .map(|step| cubelib::steps::step::StepConfig {
            kind: step.kind,
            substeps: Some(step.substeps.clone()),
            min: Some(step.min),
            max: Some(step.max),
            step_limit: None,
            quality,
            niss: Some(step.niss),
            params: step.params.clone(),
        }).collect();


    let mut cube = CubieCube::new_solved();
    cube.apply_alg(&scramble);

    let solver_steps = cubelib_solver::solver::build_steps(steps_configs, &app_data.pruning_tables).map_err(|err|error::ErrorBadRequest(err))?;
    let solutions = cubelib_solver::solver::solve_steps(cube, &solver_steps);
    let mut solutions = cubelib::stream::distinct_algorithms(solutions);

    let solution = async {
        solutions.next()
    };

    if let Ok(solution) = actix_web::rt::time::timeout(Duration::from_secs(20), solution).await {
        if let Ok(mut conn) = app_data.pool.get() {
            let insert_result = db::insert_solution(&mut conn, scramble, solution.clone(), steps.steps.clone(), quality);
            if let Err(e) = insert_result {
                error!("Failed to save solution '{}'", e);
            }
        } else {
            error!("Failed to get db connection");
        }
        info!("Cache miss");

        Ok(HttpResponse::Ok().json(solution.map(SolverResponse::from)))
    } else {
        Ok(HttpResponse::RequestTimeout().into())
    }


}
