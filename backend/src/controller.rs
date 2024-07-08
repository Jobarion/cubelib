use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use actix_web::{HttpResponse, post, Responder, web};
use actix_web_lab::body;
use base64::Engine;
use cubelib::algs::Algorithm;
use cubelib::puzzles::c333::{Cube333, Turn333};
use cubelib::puzzles::c333::steps::solver;
use cubelib::puzzles::c333::steps::tables::PruningTables333;
use cubelib::puzzles::puzzle::ApplyAlgorithm;
use cubelib::solver::CancellationToken;
use cubelib::solver::solution::Solution;
use cubelib::steps::step::StepConfig;
use cubelib_interface::{SolverRequest, SolverResponse};
use log::{error, info};

use crate::{AppData, db};

#[post("/solve_stream")]
pub async fn solve_stream(steps: web::Json<SolverRequest>, app_data: web::Data<AppData>) -> impl Responder {
    let SolverRequest{ steps, scramble } = steps.0;
    let scramble = Algorithm::from_str(scramble.as_str()).unwrap();
    let conn = app_data.pool.get();

    match conn {
        Ok(conn) => {
            if let Err(err) = db::record_request(&conn, &scramble, &steps, app_data.pruning_tables.as_ref()) {
                error!("{err}");
                return HttpResponse::InternalServerError().finish()
            }
        },
        Err(err) => {
            error!("{err}");
            return HttpResponse::InternalServerError().finish()
        }
    }

    info!("Streaming solve request for {scramble}");

    let mut cube = Cube333::default();
    cube.apply_alg(&scramble);

    let cancel_token = CancellationToken::new();
    let mut solutions = solve_steps_quality_doubling(cube, steps.clone(), app_data.pruning_tables.clone(), cancel_token.clone())
        .map(|s| Some(SolverResponse { solution: Some(s), done: false }));


    let (mut body_tx, body) = body::channel::<std::convert::Infallible>();

    let _ = web::block(move || {
        for sol in solutions {
            let data = web::Bytes::from(serde_json::to_string(&sol).unwrap());
            if let Err(_) = body_tx.send(data) {
                break;
            }
            if let Err(_) = body_tx.send(web::Bytes::from_static(b"\n")) {
                break;
            }
        }
        let data = web::Bytes::from(serde_json::to_string(&SolverResponse { solution: None, done: true }).unwrap());
        let _ = body_tx.send(data);
        let _ = body_tx.send(web::Bytes::from_static(b"\n"));
    });

    actix_rt::spawn(async move {
        actix_rt::time::sleep(Duration::from_secs(60)).await;
        cancel_token.cancel();
    });

    HttpResponse::Ok().body(body)
}

pub fn solve_steps_quality_doubling<'a>(puzzle: Cube333, steps: Vec<StepConfig>, tables: Arc<PruningTables333>, cancel_token: CancellationToken) -> impl Iterator<Item = Solution<Turn333>> {
    let mut prev_len: Option<usize> = None;
    (5..20usize).into_iter()
        .map(|q| 2u32.pow(q as u32) as usize)
        .flat_map(move |quality| {
            let mut steps = steps.clone();
            for x in &mut steps {
                x.quality = quality;
                x.step_limit = None;
            }
            let tables = tables.as_ref().clone();
            let steps = solver::build_steps(steps, &tables).unwrap();
            let best = cubelib::solver::solve_steps(puzzle, &steps, cancel_token.clone()).next();
            best
        })
        .filter(move |sol| {
            match prev_len {
                Some(p) => {
                    if sol.len() < p {
                        prev_len = Some(sol.len());
                        true
                    } else {
                        false
                    }
                },
                None => {
                    prev_len = Some(sol.len());
                    true
                }
            }
        })
}