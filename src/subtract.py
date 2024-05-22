import json
import cffi
import time
import logging
import uuid
import sys
from time import sleep


logging.basicConfig()
logging.getLogger().setLevel(logging.ERROR)


ffi = cffi.FFI()

# Define C functions from Rust
ffi.cdef("""
    int create_shared_memory();
    int write_to_input_queue(int shm_fd, const char* uuid, const char* data);
    char* read_from_input_queue(int shm_fd);
    int remove_from_input_queue(int shm_fd, const char* uuid);
    int write_to_output_queue(int shm_fd, const char* uuid, const char* data);
    char* read_from_output_queue(int shm_fd);
    int remove_from_output_queue(int shm_fd, const char* uuid);
    int clear_shared_memory(int shm_fd);
""")

# Load Rust shared library
C = ffi.dlopen("../rust_core/target/release/librust_core.so")

# Create shared memory and get the file descriptor
shm_fd = C.create_shared_memory()
if shm_fd == -1:
    raise Exception("Failed to create shared memory")

def process_message(message):
    data = json.loads(message)
    function = data.get("function")
    args = data.get("args")
    uuid = data.get("uuid")

    if function != "subtract":
        logging.info("Read other functions in add!")
        return

    a = args.get("a")
    b = args.get("b")

    if a is None or b is None:
        return

    result = a - b

    response = {
        "uuid": uuid,
        "result": result
    }

    response_str = json.dumps(response)
    C.write_to_output_queue(shm_fd, uuid.encode('utf-8'), response_str.encode('utf-8'))
    C.remove_from_input_queue(shm_fd, uuid.encode('utf-8'))

def main():
    start_time = time.time()

    # call_uuid = str(uuid.uuid4())
    # function_call = {
    #     "function": "add",
    #     "uuid": call_uuid,
    #     "args": {"a": 10, "b": 4}
    # }
    # C.write_to_input_queue(shm_fd, call_uuid.encode('utf-8'), json.dumps(function_call).encode('utf-8'))

    # print(f"Written to Queue!")


    while True:
        # if time.time() - start_time > 3:
        #     print("Timeout waiting for message")
        #     break
        message_ptr = C.read_from_input_queue(shm_fd)
        if message_ptr:
            message = ffi.string(message_ptr).decode('utf-8')
            logging.info(f"Code subtract.py recieved {message}!")
            logging.info(f"message is : {message}")
            try:
                process_message(message)
                # return 
            except Exception as e:
                logging.error(f"Caught exception processing message {message}. The exception was: {e}")
                # return 
        time.sleep(0.1)

if __name__ == "__main__":
    main()
