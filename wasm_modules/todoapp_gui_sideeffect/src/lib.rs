use TodoCore::{
    CreatedTask,
    UpdatedTask,
    CompletedTask,
    DeletedTask,
    CreatedTaskLen,
    UpdatedTaskLen,
    CompletedTaskLen,
    DeletedTaskLen,
    TaskMessage,
};

use spine::Message;

struct TodoGuiSideEffect {}

#[no_mangle]
pub extern "C" const listening_to: Vec<&'static str> = vec![CreatedTask, UpdatedTask, CompletedTask, DeletedTask];

#[no_mangle]
pub extern "C" fn copy_completed_task(msg: *const TaskMessage) {

    if msg.is_null() {
        return;
    }

    let msg = unsafe { &*msg };

    
    let func_index = lookup_func_index(allocated_on_completed_task_ref_string);
    // allocate msg (stack logic)
    // get its ptr+len
    // pass to handle_async
    handle_async(func_index, ptr, len); // func_index of completed_task
    // on_completed_task deallocates the message
}

#[no_mangle]
pub extern "C" fn on_completed_task(msg: *const TaskMessage) {
    if msg.is_null() {
        return;
    }

    let msg = unsafe { &*msg };
    // handle msg
    // drop msg
}
