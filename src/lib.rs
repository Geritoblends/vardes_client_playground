use dashmap::DashMap;

pub struct Wireplumber;

pub struct HostState {
    functions: DashMap<String, Func>,
    _instances: Vec<Instance>,
}

impl Wireplumber {
    fn new() -> Self {
        Wireplumber {}
    }

    pub fn call(&self, mut caller: Caller<'_, HostState>, fn_name: *const str, fn_name_len: i32, payload_ptr: i32, payload_len: i32) {

