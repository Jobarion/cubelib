use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, Instant};
use actix_web::{HttpResponse, post, get, Responder, web, HttpRequest};
use actix_web::web::Bytes;
use actix_web_lab::body;
use base64::Engine;
use cubelib::algs::Algorithm;
use cubelib::defs::StepKind;
use cubelib::puzzles::c333::{Cube333, Transformation333, Turn333};
use cubelib::puzzles::c333::steps::dr::coords::DRUDEOFBCoord;
use cubelib::puzzles::c333::steps::htr::coords::HTRDRUDCoord;
use cubelib::puzzles::c333::steps::solver;
use cubelib::puzzles::c333::steps::tables::PruningTables333;
use cubelib::puzzles::c333::util::DR_SUBSETS;
use cubelib::puzzles::puzzle::{ApplyAlgorithm, TransformableMut};
use cubelib::solver::CancellationToken;
use cubelib::solver::solution::Solution;
use cubelib::steps::coord::Coord;
use cubelib::steps::step::StepConfig;
use cubelib_interface::{SolverRequest, SolverResponse};
use log::{debug, error, info, trace};
use crate::{AppData, db};

#[post("/solve_stream")]
pub async fn solve_stream(req: HttpRequest, steps: web::Json<SolverRequest>, app_data: web::Data<AppData>) -> impl Responder {
    let mut hasher = DefaultHasher::new();
    let peer = req.peer_addr().unwrap();
    peer.ip().to_string().hash(&mut hasher);
    serde_json::to_string(&steps).unwrap().hash(&mut hasher);
    let hash = hasher.finish();
    let last = app_data.debounce_cache.get_with(hash, ||peer);
    if last.ip() == peer.ip() && last.port() != peer.port() {
        debug!("Debounce kill");
        return HttpResponse::ServiceUnavailable().finish()
    }

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
    let cancel_token_1 = cancel_token.clone();
    let mut solutions = solve_steps_quality_doubling(cube, steps.clone(), app_data.pruning_tables.clone(), cancel_token.clone())
        .map(|s| Some(SolverResponse { solution: Some(s), done: false }));


    let (mut body_tx, body) = body::channel::<std::convert::Infallible>();

    let _ = web::block(move || {
        let mut keepalive_tx = body_tx.clone();
        let cancel_token_1 = cancel_token.clone();
        let _ = web::block(move || {
            while !cancel_token_1.is_cancelled() {
                sleep(Duration::from_secs(1));
                trace!("Sending keepalive");
                if let Err(_) = keepalive_tx.send(web::Bytes::from_static(b" ")) {
                    info!("Stream closed, cancelling");
                    cancel_token_1.cancel();
                }
            }
        });
        for sol in solutions {
            if cancel_token.is_cancelled() {
                break;
            }
            let data = web::Bytes::from(serde_json::to_string(&sol).unwrap());
            if let Err(_) = body_tx.send(data) {
                break;
            }
            if let Err(_) = body_tx.send(web::Bytes::from_static(b"\n")) {
                break;
            }
        }
        cancel_token.cancel();
        let data = web::Bytes::from(serde_json::to_string(&SolverResponse { solution: None, done: true }).unwrap());
        let _ = body_tx.send(data);
        let _ = body_tx.send(web::Bytes::from_static(b"\n"));
    });

    actix_rt::spawn(async move {
        actix_rt::time::sleep(Duration::from_secs(60)).await;
        cancel_token_1.cancel();
    });

    HttpResponse::Ok().body(body)
}

pub fn solve_steps_quality_doubling<'a>(puzzle: Cube333, steps: Vec<StepConfig>, tables: Arc<PruningTables333>, cancel_token: CancellationToken) -> impl Iterator<Item = Solution<Turn333>> {
    let mut prev_len: Option<usize> = None;
    let t1 = tables.clone();
    (5..20usize).into_iter()
        .map(|q| 2u32.pow(q as u32) as usize)
        .flat_map(move |quality| {
            let mut steps = steps.clone();
            for x in &mut steps {
                x.quality = quality;
                x.step_limit = None;
            }
            let tables = t1.as_ref().clone();
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
        // Add comments
        .map(move |mut sol|{
            let mut cube = puzzle.clone();
            for mut step in sol.steps.iter_mut() {
                cube.apply_alg(&step.alg);
                match step.kind {
                    StepKind::DR => {
                        let mut cube = cube.clone();
                        for _ in 0..3 {
                            if DRUDEOFBCoord::from(&cube).val() == 0 {
                                break;
                            }
                            cube.transform(Transformation333::X);
                            cube.transform(Transformation333::Z);
                        }
                        let subset_id = tables.as_ref().clone().htr_subset().unwrap().get(HTRDRUDCoord::from(&cube));
                        let subset = &DR_SUBSETS[subset_id as usize];
                        step.comment = subset.to_string();
                    },
                    _ => {}
                }
            }
            sol
        })
}