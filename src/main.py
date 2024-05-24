import json
import uuid
import os
import subprocess
import cffi
import logging
import multiprocessing
from time import sleep
from datetime import datetime

logging.basicConfig()
logging.getLogger().setLevel(logging.INFO)

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

def call_function(call_function_args):
    function_call_command, function_location, function_name, args = call_function_args

    call_uuid = str(uuid.uuid4())
    function_call = {
        "function": function_name,
        "uuid": call_uuid,
        "args": args
    }
    C.write_to_input_queue(shm_fd, call_uuid.encode('utf-8'), json.dumps(function_call).encode('utf-8'))

    # print("READING")
    # message_ptr = C.read_from_input_queue(shm_fd)
    # print("READ")
    # if message_ptr:
    #     message = ffi.string(message_ptr).decode('utf-8')
    #     logging.info(f"Code add.py recieved {message}!")    # Run the corresponding function program
    #     logging.info(f"message is : {message}")

    # subprocess.call([function_call_command, function_location])

    # Wait for result
    result = None
    for i in range(0, 100):
        result_ptr = C.read_from_output_queue(shm_fd)
        if result_ptr:
            result_str = ffi.string(result_ptr).decode('utf-8')
            result_data = json.loads(result_str)
            if result_data['uuid'] == call_uuid:
                result = result_data['result']
                C.remove_from_output_queue(shm_fd, call_uuid.encode('utf-8'))
                return result
        sleep(0.1)
    raise TimeoutError(f"Timeout function '{function_name}' call. Exiting...")

if __name__ == "__main__":
    # Example usage: call the add function
    try:
        start_time = datetime.now()
        
        result = call_function(["python3", "add.py", "add", {"a": 10, "b": 4}])
        print(f"Add Result: {result}")

        result = call_function(["python3", "add.py", "add", {"a": 12, "b": 5}])
        print(f"Add Result: {result}")

        result = call_function(["python3", "add.py", "add", {"a": 9, "b": 13}])
        print(f"Add Result: {result}")


        pool = multiprocessing.Pool() 
        pool = multiprocessing.Pool(processes=6) 
        # input list 
        inputs = [["python3", "add.py", "add", {"a": 1234, "b": 0}],
                  ["python3", "add.py", "add", {"a": 2, "b": 123}],
                  ["python3", "add.py", "add", {"a": 10, "b": 75}],
                  ["python3", "add.py", "add", {"a": 3333, "b": 1111}],
                  ["python3", "add.py", "add", {"a": 101013, "b": 0}],
                  ["python3", "add.py", "add", {"a": 109, "b": 85}]] 
        outputs = pool.map(call_function, inputs) 
        # Print input list 
        print("Input: {}".format(inputs)) 
        # Print output list 
        print("Output: {}".format(outputs)) 

        print(f"The to total time taken was {datetime.now()-start_time}")
        C.clear_shared_memory(shm_fd)
    except Exception as e:
        print(e)

