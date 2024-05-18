use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures_util::{stream::StreamExt, sink::SinkExt};
use serde::{Deserialize, Serialize};
use std::ffi::{CString, CStr};
use tokio_tungstenite::tungstenite::protocol::Message;
use std::process::Command;
use std::str;
use libc::{c_char, c_int};
use std::ptr;

#[link(name = "rust_core", kind = "dylib")]
extern "C" {
    fn read_from_input_queue() -> *mut c_char;
    fn write_to_input_queue(uuid: *const c_char, data: *const c_char) -> c_int;
    fn remove_from_input_queue(uuid: *const c_char) -> c_int;
    fn read_from_output_queue() -> *mut c_char;
    fn write_to_output_queue(uuid: *const c_char, data: *const c_char) -> c_int;
    fn remove_from_output_queue(uuid: *const c_char) -> c_int;
    fn clear_shared_memory();
}

#[derive(Debug, Serialize, Deserialize)]
struct FunctionCall {
    function: String,
    uuid: String,
    args: serde_json::Value,
    command: String,
    location: String,
}

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }
}

async fn handle_connection(stream: tokio::net::TcpStream) {
    let ws_stream = accept_async(stream).await.expect("Error during the websocket handshake");
    let (mut write, mut read) = ws_stream.split();

    while let Some(message) = read.next().await {
        let msg = message.expect("Failed to read message");
        if msg.is_text() {
            let function_call: FunctionCall = serde_json::from_str(msg.to_text().unwrap()).unwrap();
            let uuid = function_call.uuid.clone();
            let data = serde_json::to_string(&function_call).unwrap();

            let uuid_cstring = CString::new(uuid.clone()).unwrap();
            let data_cstring = CString::new(data).unwrap();
            unsafe {
                write_to_input_queue(uuid_cstring.as_ptr(), data_cstring.as_ptr());
            }

            // Run the corresponding function program
            let output = Command::new(&function_call.command)
                .arg(&function_call.location)
                .arg(&serde_json::to_string(&function_call.args).unwrap())
                .output()
                .expect("Failed to execute process");

            let result_str = str::from_utf8(&output.stdout).unwrap();
            println!("result_str:{:?}", result_str);
            let response = format!(
                r#"{{"uuid": "{}", "result": {}}}"#,
                uuid,
                result_str
            );

            let response_cstring = CString::new(response.clone()).unwrap();
            unsafe {
                write_to_output_queue(uuid_cstring.as_ptr(), response_cstring.as_ptr());
            }

            // Send result back through WebSocket
            let message = Message::text(response);
            write.send(message).await.expect("Failed to send message");

            // Process the message (for example purposes, this just prints it)
            println!("Received message: {:?}", function_call);
        }
    }
}
