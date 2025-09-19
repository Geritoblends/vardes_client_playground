struct TodoCore {
    db: sled::Db
}

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
    fn get_consumer_fn_indices(ptr: *const str, len: i64) -> (*const i32, len: i32);
}

#[no_mangle]
pub extern "C" fn allocate(len: i32) -> i32 {
}

#[no_mangle]
pub extern "C" fn init() {

}


