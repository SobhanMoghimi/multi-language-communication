import json
import uuid
import os
import subprocess
import cffi
import logging
from time import sleep

ffi = cffi.FFI()

# Define C functions from Rust
ffi.cdef("""
    char* read_from_input_queue();
    int write_to_input_queue(const char* uuid, const char* data);
    int remove_from_input_queue(const char* uuid);
    char* read_from_output_queue();
    int write_to_output_queue(const char* uuid, const char* data);
    int remove_from_output_queue(const char* uuid);
    void clear_shared_memory();
""")

# Load Rust shared library
C = ffi.dlopen("/home/sobhan/codes/sobhan/repos/zahra/multi-language-communication/rust_core/target/release/librust_core.so")

def call_function(function_call_command, function_location, function_name, args):
    call_uuid = str(uuid.uuid4())
    function_call = {
        "function": function_name,
        "uuid": call_uuid,
        "args": args
    }
    C.write_to_input_queue(call_uuid.encode('utf-8'), json.dumps(function_call).encode('utf-8'))
    
    # Run the corresponding function program
    subprocess.call([function_call_command, function_location])

    # Wait for result
    result = None
    for i in range(0, 100):
        result_ptr = C.read_from_output_queue()
        if result_ptr:
            result_str = ffi.string(result_ptr).decode('utf-8')
            result_data = json.loads(result_str)
            if result_data['uuid'] == call_uuid:
                result = result_data['result']
                C.remove_from_output_queue(call_uuid.encode('utf-8'))
                return result
        sleep(0.1)
    raise TimeoutError(f"Timeout function '{function_name}' call. Exisitng...")

if __name__ == "__main__":
    # Example usage: call the add function
    # result = call_function("add", {"a": 5, "b": 3})
    # print(f"Add Result: {result}")

    # Example usage: call the subtract function
    try:
        result = call_function("python3", "add.py", "add", {"a": 10, "b": 4})
        print(f"Subtract Result: {result}")
    except Exception as e:
        print(e)
