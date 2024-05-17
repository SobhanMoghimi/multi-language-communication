// add.c

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <stdbool.h>
#include <time.h>
#include <jansson.h>
#include <dlfcn.h>

#define MAX_MESSAGE_LENGTH 1024
#define TIMEOUT_SECONDS 10

// Define function pointer types for Rust core functions
typedef char* (*read_from_input_queue_t)();
typedef int (*write_to_output_queue_t)(const char*);

// Function to load a symbol from the shared library
void* load_symbol(void* handle, const char* symbol_name) {
    void* symbol = dlsym(handle, symbol_name);
    if (!symbol) {
        fprintf(stderr, "Could not load symbol %s: %s\n", symbol_name, dlerror());
        exit(1);
    }
    return symbol;
}

// Function to process the message
void process_message(const char* message, write_to_output_queue_t write_to_output_queue) {
    // Parse the JSON message
    json_t *root;
    json_error_t error;

    root = json_loads(message, 0, &error);
    if(!root) {
        fprintf(stderr, "Error parsing JSON: %s\n", error.text);
        return;
    }

    json_t *function = json_object_get(root, "function");
    json_t *args = json_object_get(root, "args");
    json_t *uuid = json_object_get(root, "uuid");

    if(!json_is_string(function) || !json_is_object(args) || !json_is_string(uuid)) {
        fprintf(stderr, "Invalid JSON structure\n");
        json_decref(root);
        return;
    }

    const char* function_name = json_string_value(function);
    if(strcmp(function_name, "add") != 0) {
        // Not for this program
        json_decref(root);
        return;
    }

    // Get arguments
    json_t *a_json = json_object_get(args, "a");
    json_t *b_json = json_object_get(args, "b");

    if(!json_is_integer(a_json) || !json_is_integer(b_json)) {
        fprintf(stderr, "Invalid arguments\n");
        json_decref(root);
        return;
    }

    int a = (int)json_integer_value(a_json);
    int b = (int)json_integer_value(b_json);

    // Perform the addition
    int result = a + b;

    // Construct response message
    json_t *response = json_object();
    json_object_set_new(response, "uuid", json_string(json_string_value(uuid)));
    json_object_set_new(response, "result", json_integer(result));

    char *response_str = json_dumps(response, 0);
    if (!response_str) {
        fprintf(stderr, "Error creating response JSON\n");
        json_decref(root);
        json_decref(response);
        return;
    }

    // Write to output queue
    write_to_output_queue(response_str);

    free(response_str);
    json_decref(root);
    json_decref(response);
}

int main() {
    // Load the Rust core shared library
    void* handle = dlopen("./librust_core.so", RTLD_LAZY);
    if (!handle) {
        fprintf(stderr, "Could not load shared library: %s\n", dlerror());
        return 1;
    }

    read_from_input_queue_t read_from_input_queue = (read_from_input_queue_t)load_symbol(handle, "read_from_input_queue");
    write_to_output_queue_t write_to_output_queue = (write_to_output_queue_t)load_symbol(handle, "write_to_output_queue");

    time_t start_time = time(NULL);

    while (true) {
        // Check for timeout
        if (difftime(time(NULL), start_time) > TIMEOUT_SECONDS) {
            fprintf(stderr, "Timeout waiting for message\n");
            break;
        }

        // Read message from input queue
        char* message = read_from_input_queue();
        if (message != NULL && strlen(message) > 0) {
            // Process the message
            process_message(message, write_to_output_queue);

            // Reset the start time after processing a message
            start_time = time(NULL);
        }

        // Sleep for a short interval before checking for the next message
        usleep(1000);
    }

    dlclose(handle);
    return 0;
}
