use std::rc::Rc;
use leptonic::Out;
use leptos::*;

pub type RwSignalTup<T: Clone> = (Signal<T>, WriteSignal<T>);

pub fn use_local_storage<T: Clone + for<'de> leptos::server_fn::serde::Deserialize<'de> + leptos::server_fn::serde::Serialize>(key: &str, default: T) -> RwSignalTup<T>{
    let x = leptos_use::storage::use_local_storage(key, default);
    (x.0, x.1)
}

pub fn build_toggle_chain<const L: usize>(save_key: &str) -> Vec<(Signal<bool>, Callback<bool>)> {
    let signals: Vec<RwSignalTup<bool>> = (0..L).into_iter()
        .map(|x| {
            let tup = use_local_storage(format!("{save_key}-{x}").as_str(), true);
            (tup.0, tup.1)
        })
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

pub fn rw_signal_to_out<T>(rw: RwSignal<T>) -> Out<T> {
    Callback::new(move |x| rw.set(x)).into()
}