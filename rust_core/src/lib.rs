use libc::{c_char, c_int, mmap, munmap, shm_open, shm_unlink, ftruncate, O_CREAT, O_RDWR, PROT_READ, PROT_WRITE, MAP_SHARED, MAP_FAILED};
use std::ffi::{CStr, CString};
use std::ptr;
use std::slice;
use std::str;

// Define constants
const SHM_NAME: &str = "/my_shared_memory";
const SHM_SIZE: usize = 4096 * 2; // Adjusted size for both input and output queues
const MAX_ENTRIES: usize = 100;
const ENTRY_SIZE: usize = 256;

// Define the structure of a queue entry
#[repr(C)]
struct QueueEntry {
    uuid: [u8; 36+1],  // UUID as a fixed-size array
    message: [u8; ENTRY_SIZE+1],  // Message as a fixed-size array
}

// Function to create shared memory
#[no_mangle]
pub extern "C" fn create_shared_memory() -> c_int {
    unsafe {
        let shm_fd = shm_open(CString::new(SHM_NAME).unwrap().as_ptr(), O_CREAT | O_RDWR, 0o666);
        if shm_fd == -1 {
            eprintln!("Failed to open shared memory");
            return -1;
        }

        if ftruncate(shm_fd, SHM_SIZE as i64) == -1 {
            eprintln!("Failed to truncate shared memory");
            shm_unlink(CString::new(SHM_NAME).unwrap().as_ptr());
            return -1;
        }

        shm_fd
    }
}

// Function to write to a queue in shared memory
unsafe fn write_to_queue(ptr: *mut u8, uuid: *const c_char, data: *const c_char) -> c_int {
    let queue: &mut [QueueEntry] = slice::from_raw_parts_mut(ptr as *mut QueueEntry, MAX_ENTRIES);
    let uuid_str = CStr::from_ptr(uuid).to_str().unwrap();
    let data_str = CStr::from_ptr(data).to_str().unwrap();

    for entry in queue.iter_mut() {
        if entry.uuid[0] == 0 {
            // Clear the entry before copying
            entry.uuid = [0; 36+1];
            entry.message = [0; ENTRY_SIZE+1];
            
            // Copy the UUID and message, including null terminators
            ptr::copy_nonoverlapping(uuid, entry.uuid.as_mut_ptr() as *mut c_char, uuid_str.len());
            entry.uuid[uuid_str.len()] = 0; // null-terminate
            
            ptr::copy_nonoverlapping(data, entry.message.as_mut_ptr() as *mut c_char, data_str.len());
            entry.message[data_str.len()] = 0; // null-terminate
            
            // println!("Write: UUID={} Message={}", uuid_str, data_str);
            break;
        }
    }
    0
}

// Function to read from a queue in shared memory
unsafe fn read_from_queue(ptr: *const u8) -> *mut c_char {
    let queue: &[QueueEntry] = slice::from_raw_parts(ptr as *const QueueEntry, MAX_ENTRIES);
    let entry = queue.iter().find(|entry| entry.uuid[0] != 0);

    if let Some(entry) = entry {
        let message = CStr::from_bytes_with_nul_unchecked(&entry.message);
        // println!("Read: UUID={} Message={}", String::from_utf8_lossy(&entry.uuid), message.to_str().unwrap_or(""));
        return CString::new(message.to_bytes()).unwrap_unchecked().into_raw();
    } else {
        return ptr::null_mut();
    }
}

// Function to remove from a queue in shared memory
// unsafe fn remove_from_queue(ptr: *mut u8, uuid: *const c_char) -> c_int {
//     let queue: &mut [QueueEntry] = slice::from_raw_parts_mut(ptr as *mut QueueEntry, MAX_ENTRIES);
//     let uuid_str = CStr::from_ptr(uuid).to_str().unwrap();

//     for entry in queue.iter_mut() {
//         if entry.uuid.starts_with(uuid_str.as_bytes()) {
//             entry.uuid = [0; 36];
//             entry.message = [0; ENTRY_SIZE];
//             break;
//         }
//     }
//     0
// }

// Function to remove from a queue in shared memory
unsafe fn remove_from_queue(ptr: *mut u8, uuid: *const c_char) -> c_int {
    let queue: &mut [QueueEntry] = slice::from_raw_parts_mut(ptr as *mut QueueEntry, MAX_ENTRIES);
    let uuid_str = CStr::from_ptr(uuid).to_str().unwrap();

    for entry in queue.iter_mut() {
        if entry.uuid.starts_with(uuid_str.as_bytes()) {
            entry.uuid = [0; 36+1];
            entry.message = [0; ENTRY_SIZE+1];
            // println!("Removed: UUID={}", uuid_str);
            break;
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn write_to_input_queue(shm_fd: c_int, uuid: *const c_char, data: *const c_char) -> c_int {
    unsafe {
        let ptr = mmap(ptr::null_mut(), SHM_SIZE, PROT_WRITE, MAP_SHARED, shm_fd, 0);
        if ptr == MAP_FAILED {
            return -1;
        }
        let result = write_to_queue(ptr as *mut u8, uuid, data);
        munmap(ptr, SHM_SIZE);
        result
    }
}

#[no_mangle]
pub extern "C" fn read_from_input_queue(shm_fd: c_int) -> *mut c_char {
    unsafe {
        let ptr = mmap(ptr::null_mut(), SHM_SIZE, PROT_READ, MAP_SHARED, shm_fd, 0);
        if ptr == MAP_FAILED {
            return ptr::null_mut();
        }
        let result = read_from_queue(ptr as *const u8);
        munmap(ptr, SHM_SIZE);
        return result;
    }
}

// Function to remove message from input queue by UUID in shared memory
#[no_mangle]
pub extern "C" fn remove_from_input_queue(shm_fd: c_int, uuid: *const c_char) -> c_int {
    unsafe {
        let ptr = mmap(ptr::null_mut(), SHM_SIZE, PROT_WRITE, MAP_SHARED, shm_fd, 0);
        if ptr == MAP_FAILED {
            return -1;
        }

        let result = remove_from_queue(ptr as *mut u8, uuid);
        munmap(ptr, SHM_SIZE);
        result
    }
}
// Function to write to output queue in shared memory
#[no_mangle]
pub extern "C" fn write_to_output_queue(shm_fd: c_int, uuid: *const c_char, data: *const c_char) -> c_int {
    unsafe {
        let ptr = mmap(ptr::null_mut(), SHM_SIZE, PROT_WRITE, MAP_SHARED, shm_fd, 0);
        if ptr == MAP_FAILED {
            return -1;
        }

        // Write to the second half of the shared memory (output queue)
        let result = write_to_queue(ptr.add(SHM_SIZE / 2) as *mut u8, uuid, data);
        munmap(ptr, SHM_SIZE);
        result
    }
}

// Function to read from output queue in shared memory
#[no_mangle]
pub extern "C" fn read_from_output_queue(shm_fd: c_int) -> *mut c_char {
    unsafe {
        let ptr = mmap(ptr::null_mut(), SHM_SIZE, PROT_READ, MAP_SHARED, shm_fd, 0);
        if ptr == MAP_FAILED {
            return ptr::null_mut();
        }

        // Read from the second half of the shared memory (output queue)
        let result = read_from_queue(ptr.add(SHM_SIZE / 2) as *const u8);
        munmap(ptr, SHM_SIZE);
        result
    }
}

// Function to remove message from output queue by UUID in shared memory
#[no_mangle]
pub extern "C" fn remove_from_output_queue(shm_fd: c_int, uuid: *const c_char) -> c_int {
    unsafe {
        let ptr = mmap(ptr::null_mut(), SHM_SIZE, PROT_WRITE, MAP_SHARED, shm_fd, 0);
        if ptr == MAP_FAILED {
            return -1;
        }

        // Remove from the second half of the shared memory (output queue)
        let result = remove_from_queue(ptr.add(SHM_SIZE / 2) as *mut u8, uuid);
        munmap(ptr, SHM_SIZE);
        result
    }
}
// Function to clear shared memory (clear both input and output queues)
#[no_mangle]
pub extern "C" fn clear_shared_memory(shm_fd: c_int) -> c_int {
    unsafe {
        let ptr = mmap(ptr::null_mut(), SHM_SIZE, PROT_WRITE, MAP_SHARED, shm_fd, 0);
        if ptr == MAP_FAILED {
            return -1;
        }

        let input_queue: &mut [QueueEntry] = slice::from_raw_parts_mut(ptr as *mut QueueEntry, MAX_ENTRIES);
        let output_queue: &mut [QueueEntry] = slice::from_raw_parts_mut(ptr.add(SHM_SIZE / 2) as *mut QueueEntry, MAX_ENTRIES);

        for entry in input_queue.iter_mut().chain(output_queue.iter_mut()) {
            entry.uuid = [0; 36+1];
            entry.message = [0; ENTRY_SIZE+1];
        }

        munmap(ptr, SHM_SIZE);
        0
    }
}