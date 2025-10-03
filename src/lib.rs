use tokio::sync::Mutex;
use wasmtime::{Instance, Memory, Module, Table};

#[repr(C)]
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Hash)]
struct GenericId {
    bytes: [u8; 12],
}

pub struct Message<'a> {
    recipient: GenericId,
    bytes: &'a Bytes, // serde_bytes
}

pub struct HostState {
    functions: Arc<Mutex<HashMap<GenericId, Func>>>,
    network_listeners: Arc<Mutex<HashMap<GenericId, Func>>>,
    side_effects: Arc<Mutex<HashMap<GenericId, Func>>>,
    _instances: Vec<Instance>,
}

impl HostState {
    pub async fn dispatch_network(
        &self,
        mut caller: Caller<'_, HostState>,
        msg: Message<'_>,
    ) -> Result<(), anyhow::Error> {
        let network_listeners = self.network_listeners.lock().await;
        let func: TypedFunc<(i64, i64), i64> = network_listeners
            .get(msg.recipient)
            .ok_or_else(|| anyhow::anyhow!("Function not found"))?
            .typed(&caller)?;

        // Get the memory from the caller
        let memory = caller
            .get_export("memory")
            .and_then(|e| e.into_memory())
            .ok_or_else(|| anyhow::anyhow!("Failed to get memory"))?;

        // Allocate space in WASM memory
        // Option 1: Call a WASM allocator function if your module exports one
        let alloc_func: TypedFunc<i64, i64> = caller
            .get_export("allocate") // or "malloc", "alloc", etc.
            .and_then(|e| e.into_func())
            .ok_or_else(|| anyhow::anyhow!("No allocator found"))?
            .typed(&caller)?;

        let payload_len = msg.bytes.len() as i64;
        let payload_ptr = alloc_func.call(&mut caller, payload_len).await?;

        // Write the bytes to memory
        memory.write(&mut caller, payload_ptr as usize, msg.bytes)?;

        // Call the function with ptr and len
        let result = func
            .call_async(&mut caller, (payload_ptr, payload_len))
            .await?;

        // Optional: Call deallocator if needed
        // let dealloc_func: TypedFunc<(i64, i64), ()> = ...
        // dealloc_func.call_async(&mut caller, (payload_ptr, payload_len)).await?;

        Ok(result)
    }
}
