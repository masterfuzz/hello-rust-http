use std::fmt;
use std::fmt::Display;
use std::fs::File;
use std::io::prelude::*;
use std::net::TcpStream;
use std::net::TcpListener;

#[derive(Debug)]
enum HTTPStatus {
  Ok = 200,
  BadRequest = 400,
  NotFound = 404,
  BadMethod = 405,
  ServerError = 500,
}


struct HTTPResponse {
  status_code: HTTPStatus,
  body: Option<Box<String>>,
}

impl HTTPResponse {
  fn write(self, stream: &mut TcpStream) {
    let status = format!("{}", self.status_code as u16);
    let header = format!("HTTP/1.1 {} ERROR", status);
    let body = match self.body {
      Some(s) => *s,
      None => status.to_string(),
    };
    let response = format!("{}\r\n\r\n{}", header, body);
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
  }

  fn log(&self) {
    println!("Response: {}", self);
  }
}

impl Display for HTTPResponse {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "HTTPResponse<{:?}>", self.status_code)
  }
}


fn main() {
  let listener = TcpListener::bind("0.0.0.0:7878").unwrap();
  println!("Listening on 0.0.0.0:7878");

  for stream in listener.incoming() {
    let stream = stream.unwrap();
    handle_connection(stream);
  }
}

fn handle_connection(mut stream: TcpStream) {
  let mut buffer = [0; 512];

  stream.read(&mut buffer).unwrap();
  let request = String::from_utf8_lossy(&buffer[..]);

  println!("Received request:\n{}\n", request);

  let resp = get_response(&request);
  resp.log();
  resp.write(&mut stream);

  println!("Wrote response\n");
}

fn get_response(input: &str) -> HTTPResponse {
  // GET / HTTP/1.1\r\n
  if input.starts_with("GET") {
    match input.split(" ").nth(1) {
      Some(s) => load_file(s),
      None => HTTPResponse{status_code: HTTPStatus::BadRequest, body: None},
    }
  } else {
    HTTPResponse{status_code: HTTPStatus::BadMethod, body: None}
  }
}

fn load_file(path: &str) -> HTTPResponse {
  let mut corrected_path = ".".to_string();
  corrected_path = corrected_path + path;
  println!("Load file: {}", corrected_path);

  let mut file = match File::open(corrected_path) {
    Ok(f) => f,
    Err(_) => return HTTPResponse{status_code: HTTPStatus::NotFound, body: None},
  };

  let mut contents = String::new();
  match file.read_to_string(&mut contents) {
    Ok(_) => HTTPResponse{status_code: HTTPStatus::Ok, body: Some(Box::new(contents))},
    Err(_) => HTTPResponse{status_code: HTTPStatus::ServerError, body: None},
  }
}

