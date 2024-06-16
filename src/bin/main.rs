use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use rust_server::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap(); // bind to the address, (unwrap) to handle the error
    
    let pool = ThreadPool::new(4); // Thread pool to handle multiple requests

    for stream in listener.incoming().take(2) { // listen for incoming connections (take(2) to limit the number of requests)
        let stream = stream.unwrap(); // handle the error

        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    // Made the stream mutable because the read takes a mutable reference to self
    let mut buffer = [0; 1024]; // create a buffer of 1024 bytes (holds data that is read) large enough to store the basic request. (Real life you want to handle request of arbitrary size)
    stream.read(&mut buffer).unwrap(); // read the stream and store it in the buffer (populates the buffer with data from the stream)
    // println!("Request: {}", String::from_utf8_lossy(&buffer[..])); // print the buffer (converts a slice of bytes into a string including invalid utf-8 characters)

    let get = b"GET / HTTP/1.1\r\n"; // create a get request, the 'b' will gives us a byte array representing our string.
    let sleep = b"GET /sleep HTTP/1.1\r\n"; // create a sleep request

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "index.html") // if the buffer starts with the get request, return the status line and the filename
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "index.html") // if the buffer starts with the sleep request, return the status line and the filename
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html") // if the buffer does not start with the get request, return the status line and the filename
    };

    let content = fs::read_to_string(filename).unwrap(); // read the content of the file (unwrap) to handle the error
    let response = format!(
        "{}\r\nContent-Lenght: {}\r\n\r\n{}",
        status_line,
        content.len(),
        content
    ); // create a response

    stream.write(response.as_bytes()).unwrap(); // write the response to the stream (convert the response into bytes and write it to the stream)
    stream.flush().unwrap(); // flush the stream (ensures all the data is written to the connection)

    // The response must be of the following format:
    // HTTP-Version Status-Code Reason-Phrase CRLF
    // headers CRLF
    // message-body
    //
    // Example: HTTP/1.1 200 OK\r\n\r\n
}
