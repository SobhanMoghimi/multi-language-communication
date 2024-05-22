use libc::{c_char, c_int, mmap, munmap, shm_open, shm_unlink, ftruncate,
     O_CREAT, O_RDWR, PROT_READ, PROT_WRITE, MAP_SHARED, MAP_FAILED,
      pthread_mutex_t, pthread_mutexattr_t, pthread_mutex_init, pthread_mutexattr_init,
       pthread_mutexattr_setpshared, pthread_mutex_lock, pthread_mutex_unlock,
        PTHREAD_PROCESS_SHARED, sem_t, sem_init, sem_destroy, sem_wait, sem_post};
use std::ffi::{CStr, CString};
use std::ptr;
use std::slice;
use std::str;
use std::mem;

// Define constants
const SHM_NAME: &str = "/my_shared_memory";
const MAX_ENTRIES: usize = 100;
const ENTRY_SIZE: usize = 256;

// Define the structure of a queue entry
#[repr(C)]
struct QueueEntry {
    uuid: [u8; 36+1],  // UUID as a fixed-size array
    message: [u8; ENTRY_SIZE+1],  // Message as a fixed-size array
}

#[repr(C)]
struct SharedMemory {
    mutex: pthread_mutex_t,
    semaphore: sem_t,
    input_queue: [QueueEntry; MAX_ENTRIES],
    output_queue: [QueueEntry; MAX_ENTRIES],
}

impl Default for SharedMemory {
    fn default() -> Self {
        unsafe {
            let mut shared_memory: SharedMemory = mem::zeroed();

            let mut attr: pthread_mutexattr_t = mem::zeroed();
            pthread_mutexattr_init(&mut attr);
            pthread_mutexattr_setpshared(&mut attr, PTHREAD_PROCESS_SHARED);
            pthread_mutex_init(&mut shared_memory.mutex, &attr);

            sem_init(&mut shared_memory.semaphore, 1, 1);

            shared_memory
        }
    }
}

#[no_mangle]
pub extern "C" fn create_shared_memory() -> c_int {
    unsafe {
        let shm_name = CString::new(SHM_NAME).unwrap();
        let shm_fd = shm_open(shm_name.as_ptr(), O_CREAT | O_RDWR, 0o666);
        if shm_fd == -1 {
            eprintln!("Failed to open shared memory");
            return -1;
        }

        if ftruncate(shm_fd, mem::size_of::<SharedMemory>() as i64) == -1 {
            eprintln!("Failed to truncate shared memory");
            shm_unlink(shm_name.as_ptr());
            return -1;
        }

        let ptr = mmap(ptr::null_mut(), mem::size_of::<SharedMemory>(), PROT_WRITE, MAP_SHARED, shm_fd, 0);
        if ptr == MAP_FAILED {
            eprintln!("Failed to map shared memory");
            return -1;
        }

        let shared_memory: &mut SharedMemory = &mut *(ptr as *mut SharedMemory);
        *shared_memory = SharedMemory::default();

        munmap(ptr, mem::size_of::<SharedMemory>());
        shm_fd
    }
}

// Function to write to a queue in shared memory
unsafe fn write_to_queue(queue: &mut [QueueEntry], uuid: *const c_char, data: *const c_char) -> c_int {
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

            println!("Write: UUID={} Message={}", uuid_str, data_str);
            break;
        }
    }
    0
}

// Function to read from a queue in shared memory
unsafe fn read_from_queue(queue: &[QueueEntry]) -> *mut c_char {
    let entry = queue.iter().find(|entry| entry.uuid[0] != 0);

    if let Some(entry) = entry {
        // Ensure the message is null-terminated
        let message_len = entry.message.iter().position(|&x| x == 0).unwrap_or(ENTRY_SIZE);
        let message = &entry.message[..message_len];
        
        let message_cstr = match CString::new(message) {
            Ok(cstr) => cstr,
            Err(_) => {
                eprintln!("Failed to convert message to CString");
                return ptr::null_mut();
            }
        };

        println!("Read: UUID={} Message={}", String::from_utf8_lossy(&entry.uuid), message_cstr.to_str().unwrap_or(""));
        message_cstr.into_raw()
    } else {
        eprintln!("No valid entry found in the queue");
        ptr::null_mut()
    }
}

// Function to remove from a queue in shared memory
unsafe fn remove_from_queue(queue: &mut [QueueEntry], uuid: *const c_char) -> c_int {
    let uuid_str = CStr::from_ptr(uuid).to_str().unwrap();

    for entry in queue.iter_mut() {
        if entry.uuid.starts_with(uuid_str.as_bytes()) {
            entry.uuid = [0; 36+1];
            entry.message = [0; ENTRY_SIZE+1];
            println!("Removed: UUID={}", uuid_str);
            return 0; // Indicate success
        }
    }
    -1 // Indicate failure (UUID not found)
}


#[no_mangle]
pub extern "C" fn write_to_input_queue(shm_fd: c_int, uuid: *const c_char, data: *const c_char) -> c_int {
    unsafe {
        let ptr = mmap(ptr::null_mut(), mem::size_of::<SharedMemory>(), PROT_WRITE, MAP_SHARED, shm_fd, 0);
        if ptr == MAP_FAILED {
            eprintln!("Failed to map shared memory");
            return -1;
        }

        let shared_memory: &mut SharedMemory = &mut *(ptr as *mut SharedMemory);

        pthread_mutex_lock(&mut shared_memory.mutex);
        sem_wait(&mut shared_memory.semaphore);

        let result = write_to_queue(&mut shared_memory.input_queue, uuid, data);

        sem_post(&mut shared_memory.semaphore);
        pthread_mutex_unlock(&mut shared_memory.mutex);

        munmap(ptr, mem::size_of::<SharedMemory>());
        result
    }
}

// Function to write to output queue in shared memory
#[no_mangle]
pub extern "C" fn write_to_output_queue(shm_fd: c_int, uuid: *const c_char, data: *const c_char) -> c_int {
    unsafe {
        let ptr = mmap(ptr::null_mut(), mem::size_of::<SharedMemory>(), PROT_WRITE, MAP_SHARED, shm_fd, 0);
        if ptr == MAP_FAILED {
            eprintln!("Failed to map shared memory");
            return -1;
        }

        let shared_memory: &mut SharedMemory = &mut *(ptr as *mut SharedMemory);

        pthread_mutex_lock(&mut shared_memory.mutex);
        sem_wait(&mut shared_memory.semaphore);

        // Write to the second half of the shared memory (output queue)
        let result = write_to_queue(&mut shared_memory.output_queue, uuid, data);

        sem_post(&mut shared_memory.semaphore);
        pthread_mutex_unlock(&mut shared_memory.mutex);

        munmap(ptr, mem::size_of::<SharedMemory>());
        result
    }
}

#[no_mangle]
pub extern "C" fn read_from_input_queue(shm_fd: c_int) -> *mut c_char {
    unsafe {
        println!("Mapping shared memory to read from input queue...");
        let ptr = mmap(ptr::null_mut(), mem::size_of::<SharedMemory>(), PROT_READ | PROT_WRITE, MAP_SHARED, shm_fd, 0);
        if ptr == MAP_FAILED {
            eprintln!("Failed to map shared memory");
            return ptr::null_mut();
        }

        let shared_memory: &SharedMemory = &*(ptr as *const SharedMemory);

        // Lock the mutex
        let mutex_ptr = &shared_memory.mutex as *const pthread_mutex_t as *mut pthread_mutex_t;
        println!("Locking mutex for reading input queue...");
        pthread_mutex_lock(mutex_ptr);

        // Wait on the semaphore
        let sem_ptr = &shared_memory.semaphore as *const sem_t as *mut sem_t;
        println!("Waiting on semaphore for reading input queue...");
        sem_wait(sem_ptr);

        // Read from the input queue
        println!("Reading from input queue...");
        let result = read_from_queue(&shared_memory.input_queue);

        // Post the semaphore
        println!("Posting semaphore after reading input queue...");
        sem_post(sem_ptr);

        // Unlock the mutex
        println!("Unlocking mutex after reading input queue...");
        pthread_mutex_unlock(mutex_ptr);

        munmap(ptr, mem::size_of::<SharedMemory>());

        let ptr = mmap(ptr::null_mut(), mem::size_of::<SharedMemory>(), PROT_READ | PROT_WRITE, MAP_SHARED, shm_fd, 0);
        if ptr == MAP_FAILED {
            eprintln!("Failed to map shared memory");
            return ptr::null_mut();
        }

        let shared_memory: &SharedMemory = &*(ptr as *const SharedMemory);

        // Lock the mutex
        let mutex_ptr = &shared_memory.mutex as *const pthread_mutex_t as *mut pthread_mutex_t;
        pthread_mutex_lock(mutex_ptr);

        // Wait on the semaphore
        let sem_ptr = &shared_memory.semaphore as *const sem_t as *mut sem_t;
        sem_wait(sem_ptr);

        // Read from the output queue
        let result = read_from_queue(&shared_memory.input_queue);

        // Post the semaphore
        sem_post(sem_ptr);

        // Unlock the mutex
        pthread_mutex_unlock(mutex_ptr);

        munmap(ptr, mem::size_of::<SharedMemory>());

        // Check if result is null
        if result.is_null() {
            eprintln!("Read from queue returned null");
        } else {
            let message = CStr::from_ptr(result).to_str().unwrap_or("");
            println!("Read from input queue: {}", message);
        }
        result
    }
}




#[no_mangle]
pub extern "C" fn read_from_output_queue(shm_fd: c_int) -> *mut c_char {
    unsafe {
        let ptr = mmap(ptr::null_mut(), mem::size_of::<SharedMemory>(), PROT_READ | PROT_WRITE, MAP_SHARED, shm_fd, 0);
        if ptr == MAP_FAILED {
            eprintln!("Failed to map shared memory");
            return ptr::null_mut();
        }

        let shared_memory: &SharedMemory = &*(ptr as *const SharedMemory);

        // Lock the mutex
        let mutex_ptr = &shared_memory.mutex as *const pthread_mutex_t as *mut pthread_mutex_t;
        pthread_mutex_lock(mutex_ptr);

        // Wait on the semaphore
        let sem_ptr = &shared_memory.semaphore as *const sem_t as *mut sem_t;
        sem_wait(sem_ptr);

        // Read from the output queue
        let result = read_from_queue(&shared_memory.output_queue);

        // Post the semaphore
        sem_post(sem_ptr);

        // Unlock the mutex
        pthread_mutex_unlock(mutex_ptr);

        munmap(ptr, mem::size_of::<SharedMemory>());

        // Check if result is null
        if result.is_null() {
            eprintln!("Read from queue returned null");
        } else {
            let message = CStr::from_ptr(result).to_str().unwrap_or("");
            println!("Read from output queue: {}", message);
        }

        result
    }
}




#[no_mangle]
pub extern "C" fn remove_from_output_queue(shm_fd: c_int, uuid: *const c_char) -> c_int {
    unsafe {
        let ptr = mmap(ptr::null_mut(), mem::size_of::<SharedMemory>(), PROT_WRITE, MAP_SHARED, shm_fd, 0);
        if ptr == MAP_FAILED {
            eprintln!("Failed to map shared memory");
            return -1;
        }

        let shared_memory: &mut SharedMemory = &mut *(ptr as *mut SharedMemory);

        pthread_mutex_lock(&mut shared_memory.mutex);
        sem_wait(&mut shared_memory.semaphore);

        // Remove from the second half of the shared memory (output queue)
        let result = remove_from_queue(&mut shared_memory.output_queue, uuid);

        sem_post(&mut shared_memory.semaphore);
        pthread_mutex_unlock(&mut shared_memory.mutex);

        munmap(ptr, mem::size_of::<SharedMemory>());
        result
    }
}

#[no_mangle]
pub extern "C" fn remove_from_input_queue(shm_fd: c_int, uuid: *const c_char) -> c_int {
    unsafe {
        let ptr = mmap(ptr::null_mut(), mem::size_of::<SharedMemory>(), PROT_WRITE, MAP_SHARED, shm_fd, 0);
        if ptr == MAP_FAILED {
            eprintln!("Failed to map shared memory");
            return -1;
        }

        let shared_memory: &mut SharedMemory = &mut *(ptr as *mut SharedMemory);

        pthread_mutex_lock(&mut shared_memory.mutex);
        sem_wait(&mut shared_memory.semaphore);

        // Remove from the second half of the shared memory (output queue)
        let result = remove_from_queue(&mut shared_memory.input_queue, uuid);

        sem_post(&mut shared_memory.semaphore);
        pthread_mutex_unlock(&mut shared_memory.mutex);

        munmap(ptr, mem::size_of::<SharedMemory>());
        result
    }
}

#[no_mangle]
pub extern "C" fn clear_shared_memory(shm_fd: c_int) -> c_int {
    unsafe {
        println!("Mapping shared memory with size: {}", mem::size_of::<SharedMemory>());
        let ptr = mmap(ptr::null_mut(), mem::size_of::<SharedMemory>(), PROT_WRITE, MAP_SHARED, shm_fd, 0);
        if ptr == MAP_FAILED {
            eprintln!("Failed to map shared memory");
            return -1;
        }

        let shared_memory: &mut SharedMemory = &mut *(ptr as *mut SharedMemory);

        pthread_mutex_lock(&mut shared_memory.mutex);
        sem_wait(&mut shared_memory.semaphore);

        println!("Clearing input queue");
        for entry in shared_memory.input_queue.iter_mut() {
            entry.uuid = [0; 36+1];
            entry.message = [0; ENTRY_SIZE+1];
        }

        println!("Clearing output queue");
        for entry in shared_memory.output_queue.iter_mut() {
            entry.uuid = [0; 36+1];
            entry.message = [0; ENTRY_SIZE+1];
        }

        sem_post(&mut shared_memory.semaphore);
        pthread_mutex_unlock(&mut shared_memory.mutex);

        println!("Unmapping shared memory");
        if munmap(ptr, mem::size_of::<SharedMemory>()) == -1 {
            eprintln!("Failed to unmap shared memory");
            return -1;
        }

        println!("Shared memory unmapped successfully");
        0
    }
}
