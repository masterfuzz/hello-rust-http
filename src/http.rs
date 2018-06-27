extern crate std;
use std::fmt;
use std::path::PathBuf;
use std::io::prelude::*;
use std::io;
use std::fs::{self, File};
use std::net::{TcpListener, TcpStream};

#[derive(Debug)]
pub enum Status {
  Ok = 200,
  BadRequest = 400,
  NotFound = 404,
  BadMethod = 405,
  ServerError = 500,
}

pub struct Response {
  status_code: Status,
  body: Option<Box<String>>,
}

pub struct Request {
  method: Method,
  url: Option<Box<String>>,
  body: Option<Box<String>>,
}

#[derive(Debug)]
pub enum RequestError {
  BadRequest, BadMethod
}

pub enum Method {
  Get, Post, Put, Head, Delete, Patch, Options
}

impl Request {
  pub fn from_stream(stream: &mut TcpStream) -> Result<Self, RequestError> {
    let mut buffer = [0; 512];

    stream.read(&mut buffer).unwrap();
    let request = String::from_utf8_lossy(&buffer[..]);

    // for now only GET
    if request.starts_with("GET") {
      match request.split(" ").nth(1) {
        Some(s) => Ok(Request::get(s)),
        None => Err(RequestError::BadRequest),
      }
    } else {
      Err(RequestError::BadMethod)
    }
  }

  pub fn get(url: &str) -> Self {
    Request{
      method: Method::Get,
      url: Some(Box::new(String::from(url))),
      body: None
    }
  }
}

pub struct Server {
  binding: &'static str,
  pub listener: TcpListener,
}

impl Server {
  pub fn bind(binding: &'static str) -> Result<Self, std::io::Error> {
    let res = TcpListener::bind(binding);
    if res.is_ok() {
      Ok(Server{binding: binding, listener: res.ok().unwrap()})
    } else {
      Err(res.err().unwrap())
    }
  }

//  pub fn incomming(self) -> Iterator<Item=Request> {
//    self.listener.incoming().map(|mut s| Request::from_stream(s.unwrap()) )
//  }
}

impl Response {
  pub fn new(status: Status, body: Option<Box<String>>) -> Self {
    Response{status_code: status, body: body}
  }

  fn ok(body: Box<String>) -> Self {
    Response{status_code: Status::Ok, body: Some(body)}
  }

  fn bad_request() -> Self {
    Response{status_code: Status::BadRequest, body: None}
  }

  fn not_found() -> Self {
    Response{status_code: Status::NotFound, body: None}
  }

  fn bad_method() -> Self {
    Response{status_code: Status::BadMethod, body: None}
  }

  fn server_error() -> Self {
    Response{status_code: Status::ServerError, body: None}
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

impl fmt::Display for Response {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Response<{:?}>", self.status_code)
  }
}

pub trait RequestHandler {
  fn handle(self, Request) -> Response;
}

#[derive(Copy, Clone)]
pub struct FileHandler;

impl FileHandler {
  pub fn new() -> Self {
    FileHandler{}
  }
}

impl RequestHandler for FileHandler {
  fn handle(self, req: Request) -> Response {
    match req.method {
      Method::Get => 
        match req.url {
          Some(s) => load_url(&s),
          None => Response::bad_request(),
        }
      _ => Response::bad_method()
    }
  }
}

fn load_url(url: &str) -> Response {
  let mut path = PathBuf::from("./");
  path.push(url);

  if path.is_dir() {
    println!("Load index: {:?}", path);

    let mut contents = String::new();
    match load_index(path, &mut contents) {
      Ok(_) => Response::ok(Box::new(contents)),
      Err(_) => Response::server_error(),
    }
  } else if path.is_file() {
    println!("Load file: {:?}", path);

    let mut contents = String::new();
    match load_file(path, &mut contents) {
      Ok(_) => Response::ok(Box::new(contents)),
      Err(_) => Response::server_error(),
    }
  } else {
    println!("Path not found {:?}", path);
    Response::not_found()
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
