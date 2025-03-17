use std::collections::hash_map::DefaultHasher;
use std::convert::Infallible;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use actix_web::{HttpResponse, post, Responder, web, HttpRequest};
use actix_web::web::Query;
use actix_web_lab::body;
use actix_web_lab::body::Sender;
use cubelib::algs::Algorithm;
use cubelib::cube::*;
use cubelib::cube::turn::{ApplyAlgorithm, TransformableMut};
use cubelib::defs::StepKind;
use cubelib::solver::df_search::CancelToken;
use cubelib::solver::solution::Solution;
use cubelib::solver_new::{build_steps, TryRecvError};
use cubelib::steps::coord::Coord;
use cubelib::steps::dr::coords::DRUDEOFBCoord;
use cubelib::steps::htr::coords::HTRDRUDCoord;
use cubelib::steps::htr::subsets::DR_SUBSETS;
use cubelib::steps::solver;
use cubelib::steps::step::StepConfig;
use cubelib::steps::tables::PruningTables333;
use cubelib_interface::{SolverRequest, SolverResponse};
use log::{debug, error, info, trace};
use serde::Deserialize;
use crate::{AppData, db};

#[derive(Clone, Default, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SolverBackend {
    #[default]
    IterStream,
    MultiPathChannel,
}

#[derive(Deserialize)]
pub struct SolveStreamParameters {
    #[serde(default)]
    backend: SolverBackend,
}

#[post("/solve_stream")]
pub async fn solve_stream(req: HttpRequest, steps: web::Json<SolverRequest>, app_data: web::Data<AppData>, params: Query<SolveStreamParameters>) -> impl Responder {
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

    info!("Streaming solve request for {scramble} using backend {:?}", params.backend);

    let mut cube = Cube333::default();
    cube.apply_alg(&scramble);

    let cancel_token = Arc::new(CancelToken::default());

    let solutions: Box<dyn Iterator<Item = Solution> + Send> = match params.backend {
        SolverBackend::IterStream => Box::new(solve_steps_quality_doubling(cube, steps.clone(), app_data.pruning_tables.clone(), cancel_token.clone())),
        SolverBackend::MultiPathChannel => Box::new(solve_steps_quality_doubling_mpc(cube, steps.clone(), cancel_token.clone())),
    };

    let (mut body_tx, body) = body::channel::<std::convert::Infallible>();

    let ct1 = cancel_token.clone();
    let _ = web::block(move || {
        let mut keepalive_tx = body_tx.clone();
        let cancel_token = ct1.clone();
        let _ = web::block(move || {
            while !ct1.is_cancelled() {
                sleep(Duration::from_secs(1));
                trace!("Sending keepalive");
                if let Err(_) = keepalive_tx.send(web::Bytes::from_static(b" ")) {
                    info!("Stream closed, cancelling");
                    ct1.cancel();
                }
            }
        });
        let mut previous_length = usize::MAX;
        for mut sol in solutions {
            if cancel_token.is_cancelled() {
                break;
            }
            if sol.len() < previous_length {
                previous_length = sol.len();
            } else {
                continue;
            }
            add_comments(cube.clone(), &mut sol, app_data.pruning_tables.as_ref());
            let resp = SolverResponse { solution: Some(sol), done: false };
            let data = web::Bytes::from(serde_json::to_string(&resp).unwrap());
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
        cancel_token.as_ref().cancel();
    });

    HttpResponse::Ok().body(body)
}

fn add_comments(mut cube: Cube333, solution: &mut Solution, tables: &PruningTables333) {
    for step in solution.steps.iter_mut() {
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
                let subset_id = tables.htr_subset().unwrap().get(HTRDRUDCoord::from(&cube));
                let subset = &DR_SUBSETS[subset_id as usize];
                step.comment = subset.to_string();
            },
            _ => {}
        }
    }
}

pub fn solve_steps_quality_doubling<'a>(puzzle: Cube333, steps: Vec<StepConfig>, tables: Arc<PruningTables333>, cancel_token: Arc<CancelToken>) -> impl Iterator<Item = Solution> {
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
            let best = cubelib::solver::solve_steps(puzzle, &steps, cancel_token.as_ref()).next();
            best
        })
}


pub fn solve_steps_quality_doubling_mpc<'a>(puzzle: Cube333, steps: Vec<StepConfig>, cancel_token: Arc<CancelToken>) -> impl Iterator<Item = Solution> {
    (5..20usize).into_iter()
        .map(|q| 2u32.pow(q as u32) as usize)
        .flat_map(move |quality| {
            if cancel_token.is_cancelled() {
                return None;
            }
            let steps = steps.clone();
            let mut steps = cubelib::solver_new::build_steps(steps).unwrap();
            steps.apply_step_limit(quality);
            let mut worker = steps.into_worker(puzzle);
            while !cancel_token.is_cancelled() {
                match worker.try_next() {
                    Ok(sol) => {
                        return Some(sol)
                    },
                    Err(TryRecvError::Disconnected) => {
                        return None
                    },
                    Err(_) => {
                        thread::sleep(Duration::from_millis(50));
                        continue;
                    },
                };
            }
            None
        })
}