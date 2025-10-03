use thiserror::Error as ThisError;
use tokio::task;
use wasmtime::{Caller, Config, Instance, Module, Store};

#[derive(Debug, ThisError)]
enum Error {}

#[no_mangle]
pub extern "C" fn host_call_fire_and_forget(
    mut caller: Caller<'_, HostState>,
    fn_id_ptr: *const GenericId,
    payload_ptr: i64,
    payload_len: i64,
) {
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let config = Config::new()
        .wasm_threads(true)
        .async_support(true)
        .wasm64(true) // check this one
        .shared_memory(true); // check this one too
    let mut caller: Caller<'_, HostState> = Caller::new();

    let network_listener = NetworkListener::new();
    while let Some(message) = listener.bind(selected_server) {}
}
