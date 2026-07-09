use crate::{resp::{RESP, bytes_to_resp}, server::process_request};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream},
};

mod resp;
mod resp_result;
mod server;
mod storage;
mod storage_result;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // create the TCP listener
    // Redis Port: 6379
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    loop {
        // Process each incoming connection
        match listener.accept().await {
            // handle  connection
            Ok((stream, _)) => {
                // Spawn the task to take care of this connection
                tokio::spawn(handle_connection(stream));
            }
            Err(e) => {
                println!("Error: {}",e);
                continue;
            }
        }
    }
}


// main entry point for valid TCP connection
async fn handle_connection(mut stream: TcpStream) {
    // Create a buffer to host the incoming data.
    let mut buffer = [0; 512];
    println!("connection accepted");

    loop {
        // Read from the stream into the buffer
        match stream.read(&mut buffer).await {
            // if stream return some data 
            // Process the data
            Ok(size) if size != 0 => {
                // intialize the index to start at the begning of buffer
                let mut index: usize = 0;

                // process the bytes and in the buffer according to content and extract the request. And the update the index
                let request = match bytes_to_resp(&buffer[..size].to_vec(), &mut index) {
                    Ok(v) => v,
                    Err(e)=> {
                        eprintln!("Error: {}",e);
                        return ;
                    }
                };

                // proces the requet
                let response = match process_request(request) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Error parsing command: {}",e);
                        return;
                    }
                };

                // Write the response to the stream
               if let Err(e) = stream.write_all(response.to_string().as_bytes()).await {
                eprintln!("Error writing to socket {}",e);
               }

            }
            // if the stream returned no data
            // the connection has been closed
            Ok(_) => {
                println!("Connection closed");
                break;
            }
            Err(e)=> {
                println!("Error: {}", e);
                break;
            }
        }
    }
}
