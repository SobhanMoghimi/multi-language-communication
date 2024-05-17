import json
import cffi
import time
import sys
import logging

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


def process_message(message):
    print("YES")
    data = json.loads(message)
    function = data.get("function")
    args = data.get("args")
    uuid = data.get("uuid")

    print("YES22")
    if function != "add":
        logging.info("Read other functions in add!")
        return

    a = args.get("a")
    b = args.get("b")

    if a is None or b is None:
        return

    print("YES33")
    result = a + b

    response = {
        "uuid": uuid,
        "result": result
    }

    response_str = json.dumps(response)
    C.write_to_output_queue(uuid.encode('utf-8'), response_str.encode('utf-8'))

def main():
    start_time = time.time()

    while True:
        if time.time() - start_time > 10:
            print("Timeout waiting for message")
            break

        message_ptr = C.read_from_input_queue()
        print(message_ptr)
        if message_ptr:
            print("Seen")
            message = ffi.string(message_ptr).decode('utf-8')
            process_message(message)
            C.remove_from_input_queue(message_ptr)

        time.sleep(0.1)

if __name__ == "__main__":
    main()
