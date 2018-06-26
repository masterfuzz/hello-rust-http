extern crate std;
use std::fmt;
use std::path::PathBuf;
use std::io::prelude::*;
use std::io;
use std::fs::{self, File};
use std::net::TcpStream;

#[derive(Debug)]
pub enum HTTPStatus {
  Ok = 200,
  BadRequest = 400,
  NotFound = 404,
  BadMethod = 405,
  ServerError = 500,
}

pub struct HTTPResponse {
  status_code: HTTPStatus,
  body: Option<Box<String>>,
}

impl HTTPResponse {
  fn ok(body: Box<String>) -> Self {
    HTTPResponse{status_code: HTTPStatus::Ok, body: Some(body)}
  }

  fn bad_request() -> Self {
    HTTPResponse{status_code: HTTPStatus::BadRequest, body: None}
  }

  fn not_found() -> Self {
    HTTPResponse{status_code: HTTPStatus::NotFound, body: None}
  }

  fn bad_method() -> Self {
    HTTPResponse{status_code: HTTPStatus::BadMethod, body: None}
  }

  fn server_error() -> Self {
    HTTPResponse{status_code: HTTPStatus::ServerError, body: None}
  }

  pub fn write(self, stream: &mut TcpStream) -> io::Result<()> {
    let status = format!("{}", self.status_code as u16);
    let header = format!("HTTP/1.1 {} ERROR", status);
    let body = match self.body {
      Some(s) => *s,
      None => status.to_string(),
    };
    let response = format!("{}\r\n\r\n{}", header, body);
    stream.write(response.as_bytes())?;
    stream.flush()
  }

  fn log(&self) {
    println!("Response: {}", self);
  }
}

impl fmt::Display for HTTPResponse {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "HTTPResponse<{:?}>", self.status_code)
  }
}


pub fn get_response(input: &str) -> HTTPResponse {
  // GET / HTTP/1.1\r\n
  if input.starts_with("GET") {
    match input.split(" ").nth(1) {
      Some(s) => load_url(s),
      None => HTTPResponse::bad_request(),
    }
  } else {
    HTTPResponse::bad_method()
  }
}

fn load_url(url: &str) -> HTTPResponse {
  let mut path = PathBuf::from("./");
  path.push(url);

  if path.is_dir() {
    println!("Load index: {:?}", path);

    let mut contents = String::new();
    match load_index(path, &mut contents) {
      Ok(_) => HTTPResponse::ok(Box::new(contents)),
      Err(_) => HTTPResponse::server_error(),
    }
  } else if path.is_file() {
    println!("Load file: {:?}", path);

    let mut contents = String::new();
    match load_file(path, &mut contents) {
      Ok(_) => HTTPResponse::ok(Box::new(contents)),
      Err(_) => HTTPResponse::server_error(),
    }
  } else {
    println!("Path not found {:?}", path);
    HTTPResponse::not_found()
  }
}

fn load_file(path: PathBuf, contents: &mut String) -> Result<usize, io::Error> {
  let mut file = File::open(path).unwrap();
  file.read_to_string(contents)
}

fn load_index(path: PathBuf, buf: &mut String) -> Result<u16, &'static str> {
  buf.push_str(&format!(
    "<h1>Index of {0}</h1>\n<hr/><a href='{0}/..'><- parent directory</a><hr/>\n", 
    path.display())
  );

  for entry in fs::read_dir(path).unwrap() {
    if let Ok(d) = entry {
      buf.push_str(&format!("<a href='{0}'>{0}</a><br/>",
             d.path().to_str().unwrap()
            )
          );
    }
  }
  buf.push_str("\n<hr/><i>helloweb v0.1</i>");
  Ok(0)
}
