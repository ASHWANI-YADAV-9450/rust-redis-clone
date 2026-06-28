use std::{ net::{TcpListener, TcpStream}};
use std::io::{Read, Write};

fn main() {
    // create the TCP listener
    // Redis Port: 6379
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    // Process each incoming connection
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                // the connection is valid
                handle_connection(&mut stream);
            }
            Err(e)=> {
                println!("error: {}", e);
            }
        }
    }
}


// main entry point for valid TCP connection
fn handle_connection(stream: &mut TcpStream) {
    // Create a buffer to host the incoming data.
    let mut buffer = [0; 512];

    // Read from the stream into the buffer
    stream.read(&mut buffer).unwrap();

    // Hardcoded response
    let response = "+PONG\r\n";

    // write the repsonse to the stream
    stream.write(response.as_bytes()).unwrap();

    // make sure the stream is flushed
    stream.flush().unwrap();
}
