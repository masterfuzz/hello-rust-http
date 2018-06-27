mod http;
use std::thread;
use http::RequestHandler;

fn main() {
  //let listener = TcpListener::bind("0.0.0.0:7878").unwrap();
  let server = http::Server::bind("0.0.0.0:7878").unwrap();
  let handler = http::FileHandler::new();
  println!("Listening on 0.0.0.0:7878");

  for stream in server.listener.incoming() {
    if let Ok(mut stream) = stream {
      //thread::spawn(|| {
                let resp = handler.handle(http::Request::from_stream(&mut stream).unwrap());
                resp.write(&mut stream);
      //         });
    } else {
      println!("Connection error");
    }
  }
}


//fn handle_connection(mut stream: TcpStream) {
//  let mut buffer = [0; 512];
//
//  stream.read(&mut buffer).unwrap();
//  let request = String::from_utf8_lossy(&buffer[..]);
//
//  println!("Received request:\n{}\n", request);
//
//  let resp = http::get_response(&request);
//  resp.write(&mut stream).unwrap();
//
//  println!("Wrote response\n");
//}

