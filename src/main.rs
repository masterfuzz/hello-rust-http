mod http;
use std::net::TcpListener;
use std::net::TcpStream;
use std::io::prelude::*;
use std::thread;

fn main() {
  let listener = TcpListener::bind("0.0.0.0:7878").unwrap();
  println!("Listening on 0.0.0.0:7878");

  for stream in listener.incoming() {
    if let Ok(s) = stream {
      thread::spawn(|| {
                handle_connection(s);
               });
    } else {
      println!("Connection error");
    }
  }
}


fn handle_connection(mut stream: TcpStream) {
  let mut buffer = [0; 512];

  stream.read(&mut buffer).unwrap();
  let request = String::from_utf8_lossy(&buffer[..]);

  println!("Received request:\n{}\n", request);

  let resp = http::get_response(&request);
  resp.write(&mut stream).unwrap();

  println!("Wrote response\n");
}

