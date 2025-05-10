use libc::{malloc, free};

#[no_mangle]
pub extern "C" fn synapse_alloc(size: usize) -> *mut u8 {
    unsafe { malloc(size) as *mut u8 }
}

#[no_mangle]
pub extern "C" fn synapse_free(ptr: *mut u8) {
    unsafe { free(ptr as *mut libc::c_void) }
}