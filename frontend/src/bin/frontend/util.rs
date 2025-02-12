use std::rc::Rc;

use leptos::*;
use leptos_use::storage::{StorageType, UseStorageOptions};
use log::info;
use crate::AppContext;

pub type RwSignalTup<T> = (Signal<T>, WriteSignal<T>, Rc<dyn Fn()>);

pub fn use_local_storage<T: Clone + for<'de> leptos::server_fn::serde::Deserialize<'de> + leptos::server_fn::serde::Serialize>(key: &str, default: T) -> RwSignalTup<T>{
    let app_context = use_context::<AppContext>().expect("App context required");

    let namespaced_key = format!("mallard-{key}");

    let storage = if app_context.session {
        window().session_storage().unwrap().unwrap()
    } else {
        window().local_storage().unwrap().unwrap()
    };
    if storage.get_item(&namespaced_key).unwrap().is_none() {
        if let Some(val) = storage.get_item(key).unwrap() {
            let _ = storage.set_item(&namespaced_key, &val);
            let _ = storage.delete(key);
        }
    }

    let storage_opts = UseStorageOptions::default()
        .storage_type(if app_context.session {
            StorageType::Session
        } else {
            StorageType::Local
        });

    let x = leptos_use::storage::use_storage_with_options(&namespaced_key, default, storage_opts);
    // let x = leptos_use::storage::use_local_storage(key, default);
    (x.0, x.1, Rc::new(x.2))
}

pub fn build_toggle_chain<const L: usize>(save_key: &str) -> Vec<(Signal<bool>, Callback<bool>)> {
    let signals: Vec<RwSignalTup<bool>> = (0..L).into_iter()
        .map(|x| use_local_storage(format!("{save_key}-{x}").as_str(), true))
        .collect();
    let mut callbacks = vec![];
    for i in 0..signals.len() {
        let signals_0 = signals.clone();
        let signals_1 = signals.clone();
        callbacks.push((Signal::derive(move || signals_0[i].0.get()), Callback::new(move |state| {
            if state {
                for j in 0..=i {
                    signals_1[j].1.set(true);
                }
            } else {
                for j in i..signals_1.len() {
                    signals_1[j].1.set(false);
                }
            }
        })));
    }
    callbacks
}
