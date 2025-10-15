use dashmap::DashMap;
use wasmtime::{Func, Instance};

pub struct HostState {
    functions: DashMap<String, Func>,
    shared_memory: Memory,
    _instances: Vec<Instance>,
}

pub struct Wireplumber;

pub struct FuncId {
    bytes: [u8; 16]
}

impl FuncId {
    fn from_name(plugin_name: &str, func_name: &str) -> Self {
        let hash = xxhash128(plugin_name, func_name);
        let bytes = hash.to_be_bytes();
        Self {
            bytes
        }
    }
}


impl Wireplumber {
    pub fn new_host_state(engine: &Engine) -> HostState {
        // Create a shared memory
        let memory_ty = MemoryType::shared(1, 65536); // min 1 page, max 65536 pages
        let shared_memory = Memory::new(&mut store, memory_ty)
            .expect("failed to create shared memory");

        HostState {
            functions: DashMap::new(),
            shared_memory,
            _instances: Vec::new(),
        }
    }

  
    pub fn get_call_oneshot(store: &Store<HostState>) -> Func {
        Func::wrap(
            store,
            |mut caller: Caller<'_, HostState>,
             fn_id_ptr: i32,
             fn_name_len: i32,
             payload_ptr: i32,
             payload_len: i32| {

                let memory = &caller.data().shared_memory;
                let mem_data = memory.data(&caller);
                
                let fn_id_start = fn_id_ptr as usize;
                let fn_id_bytes = &mem_data[fn_id_start..fn_id_start + 16];
                let mut fn_id_array = [0u8; 16];
                fn_id_array.copy_from_slice(fn_id_bytes);
                let fn_id = FuncId { bytes: fn_id_array };
                
                let state = caller.data();
                if let Some(func_ref) = state.functions.get(&fn_id) {
                    let func = func_ref.value().clone();
                    drop(func_ref);
                    
                    // Call with the same ptr/len - no copy needed!
                    match func.typed::<(i32, i32), ()>(&caller) {
                        Ok(typed_func) => {
                            if let Err(e) = typed_func.call(&mut caller, (payload_ptr, payload_len)) {
                                eprintln!("Error calling function: {}", e);
                            }
                        }
                        Err(e) => {
                            eprintln!("Function type mismatch: {}", e);
                        }
                    }
                }
            },
        )
    }

    pub fn load_plugin(
        engine: &Engine,
        store: &mut Store<HostState>,
        plugin_name: &str,
        module_bytes: &[u8],
    ) -> Result<()> {
        // Compile the module
        let module = Module::new(engine, module_bytes)?;

        // Create a linker with the shared memory
        let mut linker = Linker::new(engine);
        let shared_memory = store.data().shared_memory.clone();
        linker.define(store, "env", "memory", shared_memory)?;

        // Add call_oneshot as an import if modules need it
        let call_oneshot = Self::get_call_oneshot(store);
        linker.define(store, "env", "call_oneshot", call_oneshot)?;

        // Instantiate the module
        let instance = linker.instantiate(store, &module)?;

        // Get all exported functions
        for export in instance.exports(store) {
            let export_name = export.name();

            // Skip memory and other non-function exports
            if let Some(func) = export.into_func() {
                // Create FuncId from plugin_name and export_name
                let func_id = FuncId::from_name(plugin_name, export_name);

                // Register the function
                store.data().functions.insert(func_id, func);

                println!("Registered function: {}::{}", plugin_name, export_name);
            }
        }

        // Store the instance to keep it alive
        store.data_mut().instances.push(instance);

        Ok(())
    }

    pub fn load_plugins(
        engine: &Engine,
        store: &mut Store<HostState>,
        plugins: &[(&str, &[u8])],  // (plugin_name, wasm_bytes)
    ) -> Result<()> {
        for (plugin_name, module_bytes) in plugins {
            Self::load_plugin(engine, store, plugin_name, module_bytes)?;
        }
        Ok(())
    }

}
        

