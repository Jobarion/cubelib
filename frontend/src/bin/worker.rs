#[cfg(feature = "wasm_solver")]
use gloo_worker::Registrable;
use log::Level;
#[cfg(feature = "wasm_solver")]
use frontend::worker::FMCSolver;

fn main() {
    wasm_log::init(wasm_log::Config::new(Level::Debug));
    console_error_panic_hook::set_once();
    #[cfg(feature = "wasm_solver")]
    FMCSolver::registrar().register();
}