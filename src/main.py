import json
import uuid
import os
import subprocess
import cffi

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

def call_function(function_name, args):
    call_uuid = str(uuid.uuid4())
    function_call = {
        "function": function_name,
        "uuid": call_uuid,
        "args": args
    }
    C.write_to_input_queue(call_uuid.encode('utf-8'), json.dumps(function_call).encode('utf-8'))
    
    # Run the corresponding function program
    if function_name == "add":
        subprocess.Popen(["./add.c"])
    elif function_name == "subtract":
        subprocess.Popen(["node", "subtract.js"])

    # Wait for result
    result = None
    while result is None:
        result_ptr = C.read_from_output_queue()
        if result_ptr:
            result_str = ffi.string(result_ptr).decode('utf-8')
            result_data = json.loads(result_str)
            if result_data['uuid'] == call_uuid:
                result = result_data['result']
                C.remove_from_output_queue(call_uuid.encode('utf-8'))
        sleep(0.1)
    return result

if __name__ == "__main__":
    # Example usage: call the add function
    # result = call_function("add", {"a": 5, "b": 3})
    # print(f"Add Result: {result}")

    # Example usage: call the subtract function
    result = call_function("subtract", {"a": 10, "b": 4})
    print(f"Subtract Result: {result}")
