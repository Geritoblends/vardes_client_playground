use dashmap::DashMap;
use wasmtime::{Func, Instance};

pub struct HostState {
    functions: DashMap<String, Func>,
    _instances: Vec<Instance>,
}

pub struct Wireplumber {
    state: HostState,
}

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
    fn new() -> Self {
        let functions: DashMap<FuncId, Func> = DashMap::new();
        let _instances: Vec<Instance> = Vec::new();
        let state = HostState {
            functions,
            _instances
        };

        Wireplumber {
            state
        }
    }

    pub fn call_oneshot(&self, mut caller: Caller<'_, HostState>, fn_id: i32, fn_name_len: i32, payload_ptr: i32, payload_len: i32) {
        // 
        unimplemented!();
    }

    pub fn get_call_oneshot(&self, store: &Store<HostState>) -> Func {
        Func::wrap(
            store,
            move |mut caller: Caller<'_, HostState>,
            fn_name_ptr: i32,
            fn_name_len: i32,
            payload_ptr: i32,
            payload_len: i32| {
                self.call(caller, fn_name_ptr, fn_name_len, payload_ptr, payload_len);
            },
        )
    }
}
        

