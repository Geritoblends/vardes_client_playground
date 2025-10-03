use once_cell::unsync::Lazy;

#[repr(C)]
struct String128 {
    bytes: [u8; 128]
}

#[repr(u8)]
enum Priority {
    Low,
    Regular,
    Urgent
}

struct Task {
    id: Bytes12Id,
    title: String,
    priority: Priority,
    completed: bool,
    created_at: u32,
}

/* struct TodoCore {
    db: DashMap<Bytes12Id, Task>,
} */

struct TodoCore {
    db: RefCell<HashMap<Bytes12Id, Rc<RefCell<Task>>>>,
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
