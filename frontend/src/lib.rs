#[cfg(feature = "wasm_solver")]
pub mod worker {
    use cubelib::cubie::CubieCube;
    use cubelib::lookup_table::TableSource;
    use cubelib::solution::Solution;
    use cubelib::solver::gen_tables;
    use cubelib::steps::step::StepConfig;
    use cubelib::tables::PruningTables;
    use gloo_worker::{HandlerId, Worker, WorkerScope};
    use leptos::spawn_local;
    use serde::{Deserialize, Serialize};

    use idb::{Error, Factory, IndexParams, KeyPath, ObjectStore, ObjectStoreParams, TransactionMode};
    use wasm_bindgen::JsValue;

    use crate::worker::WorkerResponse::{InvalidStepConfig, NoSolution, Solved, UnknownError};

    pub struct FMCSolver {
        cancel: bool,
    }

    #[derive(Clone, Serialize, Deserialize)]
    pub enum WorkerResponse {
        Solved(Solution),
        NoSolution,
        InvalidStepConfig,
        UnknownError,
    }

    impl Worker for FMCSolver {
        type Input = (CubieCube, Vec<StepConfig>);
        type Message = ();
        type Output = WorkerResponse;

        fn create(_scope: &WorkerScope<Self>) -> Self {
            Self {
                cancel: false,
            }
        }

        fn update(&mut self, _scope: &WorkerScope<Self>, _msg: Self::Message) {
            self.cancel = true;
        }

        fn received(&mut self, scope: &WorkerScope<Self>, msg: Self::Input, id: HandlerId) {
            let (cube, steps_config) = msg;
            let scope = scope.clone();
            spawn_local(async move {
                match load_pt(&steps_config).await {
                    Ok(tables) => {
                        if let Some(steps) = cubelib::solver::build_steps(steps_config, &tables).ok() {
                            let solution = cubelib::solver::solve_steps(cube, &steps).next()
                                .map_or(NoSolution, |s| Solved(s));
                            scope.respond(id, solution);
                        } else {
                            scope.respond(id, InvalidStepConfig);
                        };
                    }
                    Err(s) => {
                        log::error!("Error loading pruning table: '{s}'");
                        scope.respond(id, UnknownError);
                    }
                }
            })

        }
    }

    #[derive(Serialize, Deserialize)]
    struct PtEntry {
        id: String,
        data: serde_bytes::ByteBuf,
    }

    async fn load_pt(steps: &Vec<StepConfig>) -> Result<PruningTables, String> {
        let factory = Factory::new().map_err(|e| e.to_string())?;
        let mut open_req = factory.open("maillard", Some(PruningTables::VERSION)).map_err(|e| e.to_string())?;
        open_req.on_upgrade_needed(|event| {
            let db = event.database().unwrap();
            let mut params = ObjectStoreParams::new();
            // params.key_path(Some(KeyPath::new_single("id")));
            let store = db.create_object_store("pruning_tables", params).unwrap();

            let mut index_params = IndexParams::new();
            index_params.unique(true);
            store.create_index("entry", KeyPath::new_single("id"), Some(index_params)).unwrap();
        });
        let db = open_req.await.map_err(|e| e.to_string()).map_err(|e| e.to_string())?;
        let transaction = db.transaction(&["pruning_tables"], TransactionMode::ReadOnly).map_err(|e| e.to_string())?;
        let store = transaction.object_store("pruning_tables").map_err(|e| e.to_string())?;

        let mut pt = PruningTables::new();
        for val in store.get_all(None, None).await.map_err(|e| e.to_string())? {
            let entry: PtEntry = serde_wasm_bindgen::from_value(val).map_err(|e| e.to_string())?;
            pt.load(entry.id.as_str(), entry.data.into_vec())
        }
        transaction.done().await.map_err(|e| e.to_string())?;
        gen_tables(steps, &mut pt);
        let transaction = db.transaction(&["pruning_tables"], TransactionMode::ReadWrite).map_err(|e| e.to_string())?;
        let store = transaction.object_store("pruning_tables").map_err(|e| e.to_string())?;
        if let Some(eo) = pt.eo() {
            if eo.get_source() == TableSource::Generated {
                crate::worker::store(&store, &PtEntry {
                    id: "eo".to_string(),
                    data: serde_bytes::ByteBuf::from(eo.get_bytes())
                }).await?;
            }
        }
        if let Some(dr) = pt.dr() {
            if dr.get_source() == TableSource::Generated {
                crate::worker::store(&store, &PtEntry {
                    id: "dr".to_string(),
                    data: serde_bytes::ByteBuf::from(dr.get_bytes())
                }).await?;
            }
        }
        if let Some(htr) = pt.htr() {
            if htr.get_source() == TableSource::Generated {
                crate::worker::store(&store, &PtEntry {
                    id: "htr".to_string(),
                    data: serde_bytes::ByteBuf::from(htr.get_bytes())
                }).await?;
            }
        }
        if let Some(fr) = pt.fr() {
            if fr.get_source() == TableSource::Generated {
                crate::worker::store(&store, &PtEntry {
                    id: "fr".to_string(),
                    data: serde_bytes::ByteBuf::from(fr.get_bytes())
                }).await?;
            }
        }
        if let Some(frls) = pt.fr_leave_slice() {
            if frls.get_source() == TableSource::Generated {
                crate::worker::store(&store, &PtEntry {
                    id: "frls".to_string(),
                    data: serde_bytes::ByteBuf::from(frls.get_bytes())
                }).await?;
            }
        }
        if let Some(frfin) = pt.fr_finish() {
            if frfin.get_source() == TableSource::Generated {
                crate::worker::store(&store, &PtEntry {
                    id: "frfin".to_string(),
                    data: serde_bytes::ByteBuf::from(frfin.get_bytes())
                }).await?;
            }
        }
        transaction.done().await.map_err(|e| e.to_string())?;
        Ok(pt)
    }

    async fn store(store: &ObjectStore, entry: &PtEntry) -> Result<(), String> {
        let val = serde_wasm_bindgen::to_value(entry).map_err(|e| e.to_string())?;
        store.add(&val, Some(&JsValue::from(entry.id.clone()))).await.map_err(|e| format!("{} {e}", entry.id).to_string()).map(|_|())
    }
}
