use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::sync::{Arc, Mutex};

// Define a struct to hold shared memory data
struct SharedMemory {
    input_queue: Vec<(String, String)>,    // (UUID, message) tuple for input queue
    output_queue: Vec<(String, String)>,   // (UUID, message) tuple for output queue
}

// Mutex to protect shared memory access
static SHARED_MEMORY: Mutex<SharedMemory> = Mutex::new(SharedMemory {
    input_queue: Vec::new(),
    output_queue: Vec::new(),
});

// Function to read from input message queue
#[no_mangle]
pub extern "C" fn read_from_input_queue() -> *mut c_char {
    let mut shared_memory = SHARED_MEMORY.lock().unwrap();
    if let Some(&(ref uuid, ref data)) = shared_memory.input_queue.first() {
        let c_str = CString::new(data.clone()).unwrap();
        c_str.into_raw()
    } else {
        // Return null pointer if the input queue is empty
        std::ptr::null_mut()
    }
}

// Function to write to input message queue
#[no_mangle]
pub extern "C" fn write_to_input_queue(uuid: *const c_char, data: *const c_char) -> c_int {
    let uuid_str = unsafe { CStr::from_ptr(uuid) }.to_string_lossy().into_owned();
    let data_str = unsafe { CStr::from_ptr(data) }.to_string_lossy().into_owned();

    let mut shared_memory = SHARED_MEMORY.lock().unwrap();
    shared_memory.input_queue.push((uuid_str, data_str));

    0 // Return success code
}

// Function to remove message from input queue by UUID
#[no_mangle]
pub extern "C" fn remove_from_input_queue(uuid: *const c_char) -> c_int {
    let uuid_str = unsafe { CStr::from_ptr(uuid) }.to_string_lossy().into_owned();
    let mut shared_memory = SHARED_MEMORY.lock().unwrap();
    shared_memory.input_queue.retain(|&(ref id, _)| id != &uuid_str);
    0 // Return success code
}

// Function to read from output message queue
#[no_mangle]
pub extern "C" fn read_from_output_queue() -> *mut c_char {
    let mut shared_memory = SHARED_MEMORY.lock().unwrap();
    if let Some(&(ref uuid, ref data)) = shared_memory.output_queue.first() {
        let c_str = CString::new(data.clone()).unwrap();
        c_str.into_raw()
    } else {
        // Return null pointer if the output queue is empty
        std::ptr::null_mut()
    }
}

// Function to write to output message queue
#[no_mangle]
pub extern "C" fn write_to_output_queue(uuid: *const c_char, data: *const c_char) -> c_int {
    let uuid_str = unsafe { CStr::from_ptr(uuid) }.to_string_lossy().into_owned();
    let data_str = unsafe { CStr::from_ptr(data) }.to_string_lossy().into_owned();

    let mut shared_memory = SHARED_MEMORY.lock().unwrap();
    shared_memory.output_queue.push((uuid_str, data_str));

    0 // Return success code
}

// Function to remove message from output queue by UUID
#[no_mangle]
pub extern "C" fn remove_from_output_queue(uuid: *const c_char) -> c_int {
    let uuid_str = unsafe { CStr::from_ptr(uuid) }.to_string_lossy().into_owned();
    let mut shared_memory = SHARED_MEMORY.lock().unwrap();
    shared_memory.output_queue.retain(|&(ref id, _)| id != &uuid_str);
    0 // Return success code
}

// Function to clear shared memory (clear both input and output queues)
#[no_mangle]
pub extern "C" fn clear_shared_memory() {
    let mut shared_memory = SHARED_MEMORY.lock().unwrap();
    shared_memory.input_queue.clear();
    shared_memory.output_queue.clear();
}
