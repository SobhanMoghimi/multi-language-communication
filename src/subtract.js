const ffi = require('ffi-napi');
const ref = require('ref-napi');
const Struct = require('ref-struct-di')(ref);

// Define the Rust core library interface
const RustCore = ffi.Library('./librust_core.so', {
    'read_from_input_queue': ['pointer', []],
    'remove_from_input_queue': ['int', ['string']],
    'write_to_output_queue': ['int', ['string']]
});

// Define a struct to hold message data
const Message = Struct({
    uuid: 'string',
    function: 'string',
    args: 'string'
});

// Function to process the message
function processMessage(message) {
    try {
        // Parse the JSON message
        const { uuid, args } = JSON.parse(message);

        // Parse args
        const { minuend, subtrahend } = JSON.parse(args);

        // Perform the subtraction
        const result = minuend - subtrahend;

        // Construct response message
        const response = {
            uuid,
            result
        };

        // Write the response to the output queue
        RustCore.write_to_output_queue(JSON.stringify(response));
    } catch (error) {
        console.error('Error processing message:', error);
    }
}

// Main function to read and process messages
function main() {
    const startTime = Date.now();
    const timeout = 10000; // 10 seconds timeout

    while (Date.now() - startTime < timeout) {
        // Read message from input queue
        const messagePtr = RustCore.read_from_input_queue();
        if (!ref.isNull(messagePtr)) {
            const message = ref.readCString(messagePtr);

            // Parse the message
            const { function: func } = JSON.parse(message);

            // Check if the message is for the subtract function
            if (func === 'subtract') {
                // Process the message
                processMessage(message);

                // Remove message from input queue
                RustCore.remove_from_input_queue(JSON.parse(message).uuid);

                // Exit the loop after processing the message
                break;
            }
        }

        // Wait a bit before reading the next message
        // Note: This is a simple approach. A better approach would involve using a synchronization mechanism
        // such as a semaphore or event to wait for a new message.
        // For demonstration purposes, a simple sleep is used here.
        // Avoid using a busy loop in a real-world application.
        // Sleep for 100 milliseconds
        const sleepTime = 100;
        Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, sleepTime);
    }

    console.log('Timeout reached or message processed.');
}

// Call the main function
main();