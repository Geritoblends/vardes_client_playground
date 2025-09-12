use arc_swap::ArcSwap;

pub struct Message<const N: usize> {
    bytes: [u8; N],
    len: u32
}

pub struct Bytes12ID {
    bytes: [u8; 12]
}

pub trait SideEffect {
    fn copy_msg(&mut self, bytes: &[u8], len: usize);
}

pub trait Core {
    fn load();
    fn get_fn_address_list() -> Vec<Bytes12ID>;
}

fn lazy_load_core_function(core: Core) -> CoreFunction {
    unimplemented()!;
}

pub struct CoreFunction {
    raw_pointer_fn: *const fn(Message) -> Response,
}
    
pub struct PluginEngine {
    _cores: ArcSwap<HashMap<Bytes12ID, Core>>,
    core_functions: ArcSwap<HashMap<Bytes12ID, CoreFunction>>,
    side_effects: ArcSwap<HashMap<Bytes12ID, Box<dyn SideEffect>>>
}

// SideEffects read messages and call Core methods
// Cores provide direct method calls and broadcast messages
//
// Pending to know: 
// 1. How will Response look
// 2. How will Message exactly look
// 3. How will each SideEffect, and Core be obtained through both: WASM and dynlibs
