pub struct TodoCore {
    db: sled::Db
}

static TODO_CORE: Lazy<Mutex<Option<TodoCore>>> = Lazy::new(|| Mutex::new(None));

#[repr(C)]
pub struct TaskMessage {
    id: Bytes12Id,
    user_id: Bytes12Id,
    title: Bytes128String,
    priority: u8,
    completed: bool
}

extern "C" {
    fn lookup_func_index(ptr: i64, len: i64) -> i64;
    fn get_consumer_fn_indices(ptr: *const str, len: i64) -> (*const i64, len: i64);
}

#[no_mangle]
pub extern "C" fn allocate(len: i32) -> i32 {
}

#[no_mangle]
pub extern "C" fn init() {
    let db = sled::Db::new();
    let core = TodoCore { db };
}

#[no_mangle]
pub extern "C" fn add_item(ptr: *mut TodoCore, item_ptr: *const Task) -> i32 {
    unimplemented!();
}
