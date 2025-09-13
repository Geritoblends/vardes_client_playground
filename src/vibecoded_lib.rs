use arc_swap::ArcSwap;
use std::collections::HashMap;
use std::ffi::c_void;

// Your existing types
#[repr(C)]
pub struct Message<const N: usize> {
    bytes: [u8; N],
    len: u32,
}

#[repr(C)]
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct Bytes12ID {
    bytes: [u8; 12],
}

#[repr(C)]
pub struct Response {
    data: *const u8,
    len: u32,
    // Add error code or success indicator
    status: i32,
}

// Core abstraction - this will be implemented differently for WASM vs dynlib
pub trait Core: Send + Sync {
    fn load(&mut self) -> Result<(), String>;
    fn get_fn_address_list(&self) -> Vec<Bytes12ID>;
    fn call_function(&self, fn_id: &Bytes12ID, message: &[u8]) -> Result<Response, String>;
    fn shutdown(&mut self);
}

pub trait SideEffect: Send + Sync {
    fn copy_msg(&mut self, bytes: &[u8], len: usize);
    fn get_id(&self) -> Bytes12ID;
}

// Unified function call interface
pub struct CoreFunction {
    core_id: Bytes12ID,
    fn_id: Bytes12ID,
    // Store the execution context type
    exec_type: ExecutionType,
}

#[derive(Clone, Debug)]
pub enum ExecutionType {
    Wasm,
    DynLib,
}

impl CoreFunction {
    pub fn call(&self, engine: &PluginEngine, message: &[u8]) -> Result<Response, String> {
        let cores = engine._cores.load();
        if let Some(core) = cores.get(&self.core_id) {
            core.call_function(&self.fn_id, message)
        } else {
            Err("Core not found".to_string())
        }
    }
}

// WASM-specific implementation
pub struct WasmCore {
    id: Bytes12ID,
    instance: Option<wasmtime::Instance>,
    store: Option<wasmtime::Store<()>>,
    engine: wasmtime::Engine,
    // Memory for host-guest communication
    shared_memory: Option<wasmtime::Memory>,
    // Function exports cache
    functions: HashMap<Bytes12ID, wasmtime::TypedFunc<(i32, i32), i32>>,
}

impl WasmCore {
    pub fn new(id: Bytes12ID, wasm_bytes: &[u8]) -> Result<Self, String> {
        let engine = wasmtime::Engine::default();
        let module = wasmtime::Module::new(&engine, wasm_bytes)
            .map_err(|e| format!("Failed to create WASM module: {}", e))?;
        
        let mut store = wasmtime::Store::new(&engine, ());
        let instance = wasmtime::Instance::new(&mut store, &module, &[])
            .map_err(|e| format!("Failed to instantiate WASM module: {}", e))?;

        // Get the linear memory
        let memory = instance.get_memory(&mut store, "memory")
            .ok_or("WASM module must export 'memory'")?;

        Ok(WasmCore {
            id,
            instance: Some(instance),
            store: Some(store),
            engine,
            shared_memory: Some(memory),
            functions: HashMap::new(),
        })
    }

    fn write_to_memory(&mut self, data: &[u8]) -> Result<i32, String> {
        let store = self.store.as_mut().ok_or("Store not available")?;
        let memory = self.shared_memory.as_ref().ok_or("Memory not available")?;
        
        // Allocate memory in WASM module by calling its allocator
        let alloc_fn = self.instance.as_ref().unwrap()
            .get_typed_func::<i32, i32>(store, "alloc")
            .map_err(|e| format!("WASM module must export 'alloc' function: {}", e))?;
        
        let ptr = alloc_fn.call(store, data.len() as i32)
            .map_err(|e| format!("Failed to allocate memory in WASM: {}", e))?;
        
        // Write data to allocated memory
        let memory_data = memory.data_mut(store);
        if ptr as usize + data.len() > memory_data.len() {
            return Err("Insufficient memory in WASM module".to_string());
        }
        
        memory_data[ptr as usize..(ptr as usize + data.len())].copy_from_slice(data);
        Ok(ptr)
    }

    fn read_from_memory(&self, ptr: i32, len: i32) -> Result<Vec<u8>, String> {
        let store = self.store.as_ref().ok_or("Store not available")?;
        let memory = self.shared_memory.as_ref().ok_or("Memory not available")?;
        
        let memory_data = memory.data(store);
        if ptr < 0 || len < 0 || (ptr as usize + len as usize) > memory_data.len() {
            return Err("Invalid memory access".to_string());
        }
        
        Ok(memory_data[ptr as usize..(ptr as usize + len as usize)].to_vec())
    }
}

impl Core for WasmCore {
    fn load(&mut self) -> Result<(), String> {
        // Cache all exported functions
        let instance = self.instance.as_ref().ok_or("Instance not available")?;
        let mut store = self.store.as_mut().ok_or("Store not available")?;
        
        // Get function list from WASM module
        let get_fn_list = instance
            .get_typed_func::<(), i32>(&mut store, "get_fn_list")
            .map_err(|e| format!("WASM module must export 'get_fn_list': {}", e))?;
        
        let fn_list_ptr = get_fn_list.call(&mut store, ())
            .map_err(|e| format!("Failed to get function list: {}", e))?;
        
        // Read function IDs from memory (assuming they return a serialized list)
        // This is simplified - you'd need to define the serialization format
        
        Ok(())
    }

    fn get_fn_address_list(&self) -> Vec<Bytes12ID> {
        self.functions.keys().cloned().collect()
    }

    fn call_function(&self, fn_id: &Bytes12ID, message: &[u8]) -> Result<Response, String> {
        let mut store = self.store.as_ref().ok_or("Store not available")?.clone();
        
        // Write message to WASM memory
        let mut core_mut = unsafe { &mut *(self as *const _ as *mut WasmCore) };
        let msg_ptr = core_mut.write_to_memory(message)?;
        
        // Get and call the function
        if let Some(func) = self.functions.get(fn_id) {
            let result_ptr = func.call(&mut store, (msg_ptr, message.len() as i32))
                .map_err(|e| format!("WASM function call failed: {}", e))?;
            
            // Read response from memory (you'd need to define the response format)
            // For now, assuming the WASM function returns a pointer to Response struct
            let response_data = self.read_from_memory(result_ptr, std::mem::size_of::<Response>() as i32)?;
            
            // Deserialize response - this is simplified
            Ok(Response {
                data: std::ptr::null(),
                len: 0,
                status: 0,
            })
        } else {
            Err("Function not found".to_string())
        }
    }

    fn shutdown(&mut self) {
        self.instance = None;
        self.store = None;
        self.functions.clear();
    }
}

// Dynamic library implementation
pub struct DynLibCore {
    id: Bytes12ID,
    lib: Option<libloading::Library>,
    functions: HashMap<Bytes12ID, libloading::Symbol<'static, extern "C" fn(*const u8, u32) -> Response>>,
}

impl DynLibCore {
    pub fn new(id: Bytes12ID, lib_path: &str) -> Result<Self, String> {
        let lib = unsafe { libloading::Library::new(lib_path) }
            .map_err(|e| format!("Failed to load dynamic library: {}", e))?;
        
        Ok(DynLibCore {
            id,
            lib: Some(lib),
            functions: HashMap::new(),
        })
    }
}

impl Core for DynLibCore {
    fn load(&mut self) -> Result<(), String> {
        let lib = self.lib.as_ref().ok_or("Library not loaded")?;
        
        // Get function list from dynlib
        let get_fn_list: libloading::Symbol<extern "C" fn() -> *const Bytes12ID> = unsafe {
            lib.get(b"get_fn_list")
                .map_err(|e| format!("Failed to get function list: {}", e))?
        };
        
        let fn_list_ptr = get_fn_list();
        // Read function IDs - you'd need to define how many functions there are
        
        Ok(())
    }

    fn get_fn_address_list(&self) -> Vec<Bytes12ID> {
        self.functions.keys().cloned().collect()
    }

    fn call_function(&self, fn_id: &Bytes12ID, message: &[u8]) -> Result<Response, String> {
        if let Some(func) = self.functions.get(fn_id) {
            let response = func(message.as_ptr(), message.len() as u32);
            Ok(response)
        } else {
            Err("Function not found".to_string())
        }
    }

    fn shutdown(&mut self) {
        self.lib = None;
        self.functions.clear();
    }
}

// Message broadcasting system
pub struct MessageBroadcaster {
    side_effects: ArcSwap<HashMap<Bytes12ID, Box<dyn SideEffect>>>,
}

impl MessageBroadcaster {
    pub fn new() -> Self {
        Self {
            side_effects: ArcSwap::new(Arc::new(HashMap::new())),
        }
    }

    pub fn broadcast(&self, message: &[u8]) {
        let side_effects = self.side_effects.load();
        for side_effect in side_effects.values() {
            // Clone the side effect for thread safety if needed
            // This is a simplified version - you might want to use channels or other mechanisms
        }
    }

    pub fn register_side_effect(&self, side_effect: Box<dyn SideEffect>) {
        let id = side_effect.get_id();
        let current = self.side_effects.load();
        let mut new_map = (**current).clone();
        new_map.insert(id, side_effect);
        self.side_effects.store(Arc::new(new_map));
    }
}

// Updated PluginEngine
pub struct PluginEngine {
    cores: ArcSwap<HashMap<Bytes12ID, Box<dyn Core>>>,
    core_functions: ArcSwap<HashMap<Bytes12ID, CoreFunction>>,
    broadcaster: MessageBroadcaster,
}

impl PluginEngine {
    pub fn new() -> Self {
        Self {
            cores: ArcSwap::new(Arc::new(HashMap::new())),
            core_functions: ArcSwap::new(Arc::new(HashMap::new())),
            broadcaster: MessageBroadcaster::new(),
        }
    }

    pub fn load_wasm_core(&self, id: Bytes12ID, wasm_bytes: &[u8]) -> Result<(), String> {
        let mut core = WasmCore::new(id.clone(), wasm_bytes)?;
        core.load()?;
        
        // Register core functions
        let fn_list = core.get_fn_address_list();
        for fn_id in fn_list {
            let core_fn = CoreFunction {
                core_id: id.clone(),
                fn_id: fn_id.clone(),
                exec_type: ExecutionType::Wasm,
            };
            
            let current_fns = self.core_functions.load();
            let mut new_fns = (**current_fns).clone();
            new_fns.insert(fn_id, core_fn);
            self.core_functions.store(Arc::new(new_fns));
        }
        
        // Register core
        let current_cores = self.cores.load();
        let mut new_cores = (**current_cores).clone();
        new_cores.insert(id, Box::new(core));
        self.cores.store(Arc::new(new_cores));
        
        Ok(())
    }

    pub fn load_dynlib_core(&self, id: Bytes12ID, lib_path: &str) -> Result<(), String> {
        let mut core = DynLibCore::new(id.clone(), lib_path)?;
        core.load()?;
        
        // Similar registration process as WASM
        let fn_list = core.get_fn_address_list();
        for fn_id in fn_list {
            let core_fn = CoreFunction {
                core_id: id.clone(),
                fn_id: fn_id.clone(),
                exec_type: ExecutionType::DynLib,
            };
            
            let current_fns = self.core_functions.load();
            let mut new_fns = (**current_fns).clone();
            new_fns.insert(fn_id, core_fn);
            self.core_functions.store(Arc::new(new_fns));
        }
        
        let current_cores = self.cores.load();
        let mut new_cores = (**current_cores).clone();
        new_cores.insert(id, Box::new(core));
        self.cores.store(Arc::new(new_cores));
        
        Ok(())
    }

    pub fn call_function(&self, fn_id: &Bytes12ID, message: &[u8]) -> Result<Response, String> {
        let functions = self.core_functions.load();
        if let Some(core_fn) = functions.get(fn_id) {
            let result = core_fn.call(self, message)?;
            
            // Broadcast the message to side effects
            self.broadcaster.broadcast(message);
            
            Ok(result)
        } else {
            Err("Function not found".to_string())
        }
    }

    pub fn register_side_effect(&self, side_effect: Box<dyn SideEffect>) {
        self.broadcaster.register_side_effect(side_effect);
    }
}

// Helper function for lazy loading
pub fn lazy_load_core_function(engine: &PluginEngine, fn_id: &Bytes12ID) -> Option<&CoreFunction> {
    let functions = engine.core_functions.load();
    functions.get(fn_id)
}
