use std::rc::Rc;

use leptos::*;
use leptos_use::storage::{StorageType, UseStorageOptions};

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
