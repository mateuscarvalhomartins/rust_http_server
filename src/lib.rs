use std::{thread, time, collections::HashMap, fs::{File, read_dir}, io::*,
          net::{TcpListener, TcpStream, SocketAddr}, ffi::OsStr,};

fn header_to_string(header: HashMap<String, String>) -> String {
  let mut s = String::new();
  for (k, v) in header {
    s.push_str(&k);
    s.push_str(": ");
    s.push_str(&v);
    s.push_str("\r\n");
  }
  s
}
fn header_from_string(s: &str) -> HashMap<String, String> {
  let mut header: HashMap<String, String> = HashMap::new();
  let str_header = &s[s.find("\r\n").unwrap_or(s.len())+1..s.find("\r\n\r\n").unwrap_or(s.len())];
  let split_header = str_header.split("\r\n");
  for str in split_header {
    let split_str = str.split(": ");
    let vec_str = split_str.collect::<Vec<&str>>();
    if vec_str.len() > 1 {
      if vec_str[0].rfind("\n") != None {
        header.insert(vec_str[0][vec_str[0].rfind("\n").unwrap()+1..].to_string(), vec_str[1].to_string());
      } else {
        header.insert(vec_str[0].to_string(), vec_str[1].to_string());
      }
    }
  }
  header
}

fn body_to_string(header: HashMap<String, String>) -> String {
  let mut s = String::new();
  for (k, v) in header {
    s.push_str(&k);
    s.push_str("=");
    s.push_str(&v);
    s.push_str("&");
  }
  s
}

fn get_content_type(file_ext: &str) -> Option<(String, String)> {
  match file_ext.to_lowercase().as_str() {
    "html" => Some(("text/html; charset=utf-8".to_string(), "text/html".to_string())),
    "css" => Some(("text/css; charset=utf-8".to_string(), "text/css".to_string())),
    "js" => Some(("text/javascript; charset=utf-8".to_string(), "text/javascript".to_string())),
    "png" => Some(("image/png".to_string(), "image/png".to_string())),
    "jpg" => Some(("image/jpeg".to_string(), "image/jpeg".to_string())),
    "jpeg" => Some(("image/jpeg".to_string(), "image/jpeg".to_string())),
    "gif" => Some(("image/gif".to_string(), "image/gif".to_string())),
    "ico" => Some(("image/x-icon".to_string(), "image/x-icon".to_string())),
    "svg" => Some(("image/svg+xml".to_string(), "image/svg+xml".to_string())),
    "mid" => Some(("audio/midi".to_string(), "audio/midi".to_string())),
    "mp3" => Some(("audio/mpeg".to_string(), "audio/mpeg".to_string())),
    "wav" => Some(("audio/wav".to_string(), "audio/wav".to_string())),
    "mp4" => Some(("video/mp4".to_string(), "video/mp4".to_string())),
    _ => None
  }
}

pub fn read_file(mut file: File) -> String {
  let mut contents = String::new();
  match file.read_to_string(&mut contents) {
    Ok(_) => contents,
    Err(_) => String::new(),
  }
}

pub struct Uri {
  pub path: String,
  pub query: Option<HashMap<String, String>>
}
impl Clone for Uri {
  fn clone(&self) -> Self {
    Uri {
      path: self.path.clone(),
      query: self.query.clone()
    }
  }
}
impl ToString for Uri {
  fn to_string(&self) -> String {
    if self.query.is_some() {
      format!("{}?{}", self.path, header_to_string(self.query.clone().unwrap()))
    } else {
      self.path.clone()
    }
  }
}
impl From<&str> for Uri {
  fn from(s: &str) -> Self {
    let mut query: Option<HashMap<String, String>> = None;

    if s.find("?") != None {
      query = Some(HashMap::new());
      let params_query_not_splitted = &s[s.find("?").unwrap()+1..s.len()].split("&");
      for param in params_query_not_splitted.clone().into_iter() {
        let mut split = param.split("=");
        let k = split.next().unwrap().to_string();
        let v = split.next().expect(&format!("{} has no value", k)).to_string();
        query.as_mut().unwrap().insert(k, v);
      }
    }

    Uri {
      path: (*(&s[0..s.find("?").unwrap_or(s.len())].to_string()).clone()).to_string(),
      query: query
    }
  }
}
impl Default for Uri {
  fn default() -> Self {
    Uri { path: "/".to_string(), query: None }
  }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum Method {
  GET,
  POST,
  PUT,
  DELETE,
  HEAD,
  CONNECT,
  OPTIONS,
  TRACE,
  PATCH
}
impl ToString for Method {
  fn to_string(&self) -> String {
    match *self {
      Method::GET => format!("GET"),
      Method::POST => format!("POST"),
      Method::PUT => format!("PUT"),
      Method::DELETE => format!("DELETE"),
      Method::HEAD => format!("HEAD"),
      Method::CONNECT => format!("CONNECT"),
      Method::OPTIONS => format!("OPTIONS"),
      Method::TRACE => format!("TRACE"),
      Method::PATCH => format!("PATCH")
    }
  }
}
impl From<&str> for Method {
  fn from(s: &str) -> Self {
    match s {
      "GET" => Method::GET,
      "POST" => Method::POST,
      "PUT" => Method::PUT,
      "DELETE" => Method::DELETE,
      "HEAD" => Method::HEAD,
      "CONNECT" => Method::CONNECT,
      "OPTIONS" => Method::OPTIONS,
      "TRACE" => Method::TRACE,
      "PATCH" => Method::PATCH,
      _ => Method::GET
    }
  }
}

pub struct Request {
  pub method: Method,
  pub uri: Uri,
  pub protocolo: String,
  pub header: HashMap<String, String>,
  pub body: Option<HashMap<String, String>>
}
impl ToString for Request {
  fn to_string(&self) -> String {
    format!("{} {} {}\r\n{}\r\n\r\n{}", self.method.to_string(), self.uri.to_string(), self.protocolo, header_to_string(self.header.clone()), body_to_string(self.body.clone().unwrap_or(HashMap::new())))
  }
}
impl Clone for Request {
  fn clone(&self) -> Self {
    Request {
      uri: self.uri.clone(),
      method: self.method.clone(),
      protocolo: self.protocolo.clone(),
      header: self.header.clone(),
      body: self.body.clone()
    }
  }
}
impl From<&str> for Request {
  fn from(s: &str) -> Self {
    let str_status_line = &s[..s.find("\r\n").expect("Request has no header")];
    let split_status_line = str_status_line.split(" ");
    let vec_status_line = split_status_line.collect::<Vec<&str>>();

    let header: HashMap<String, String> = header_from_string(s);

    let mut body: HashMap<String, String> = HashMap::new();
    if s.find("\r\n\r\n") != None {
      let str_body = &s[s.find("\r\n\r\n").unwrap()+1..];
      let split_body = str_body.split("&");
      for str in split_body {
        let split_str = str.split("=");
        let vec_str = split_str.collect::<Vec<&str>>();
        if vec_str.len() > 1 {
          body.insert(vec_str[0].to_string(), vec_str[1].to_string());
        }
      }
    }

    Request { method: Method::from(vec_status_line[0]), uri: Uri::from(vec_status_line[1]), protocolo: vec_status_line[2].to_string(),
              header: header, body: Some(body) }
  }
}
impl Default for Request {
  fn default() -> Self {
    Request { method: Method::GET, uri: Uri::default(), protocolo: "HTTP/1.1".to_string(), header: HashMap::new(), body: None }
  }
}

pub struct Response {
  pub protocolo: String,
  pub status: String,
  pub header: HashMap<String, String>,
  pub body: String
}
impl ToString for Response {
  fn to_string(&self) -> String {
    format!("{} {}\r\n{}\r\n\r\n{}", self.protocolo, self.status,
            header_to_string(self.header.clone()),
            self.body)
  }
}
impl Clone for Response {
  fn clone(&self) -> Self {
    Response {
      protocolo: self.protocolo.clone(),
      status: self.status.clone(),
      header: self.header.clone(),
      body: self.body.clone()
    }
  }
}
impl From<&str> for Response {
  fn from(s: &str) -> Self {
    let protocolo = &s[..s.find(" ").expect("Response has no protocolo")];
    let status = &s[s.find(" ").unwrap()+1..s.find("\r\n").unwrap_or(s.find("\r\n\r\n").expect("Response has an empty body"))];
    let header = header_from_string(s);
    let body = &s[s.find("\r\n\r\n").unwrap()..];
    Response { protocolo: protocolo.to_string(), status: status.to_string(), header: header, body: body.to_string() }
  }
}

struct Context {
  pub is_static: bool,
  pub handler: Option<&'static (dyn Fn(&Request) -> Response + Sync)>,
  pub static_response: Option<Response>
}
impl Clone for Context {
  fn clone(&self) -> Self {
    Context {
      is_static: self.is_static.clone(),
      handler: self.handler.clone(),
      static_response: self.static_response.clone()
    }
  }
}

pub struct HttpServer {
  contexts: HashMap<(Method, String), Context>
}
impl HttpServer {
  pub fn new() -> Self {
    HttpServer {
      contexts: HashMap::new(),
    }
  }
  pub fn clone(&self) -> Self {
    HttpServer {
      contexts: self.contexts.clone(),
    }
  }

  pub fn add_context_handler(&mut self, method: Method, at: &str, handler: &'static (dyn Fn(&Request) -> Response + Sync)) {
    self.contexts.insert((method, at.to_string()), Context { is_static: false, static_response: None, handler: Some(handler) });
  }

  pub fn add_static_files(&mut self, dir_path: &str, at: &str) {
    let directory = match read_dir(&dir_path) {
      Ok(d) => d,
      Err(e) => {
        eprintln!("Problem reading directory: {}\r\nErr: {}", dir_path, e);
        return
      }
    };
    if at.is_empty() {
      for result_path in directory {
        let path = result_path.unwrap();
        self.contexts.insert((Method::GET, format!("/{}", path.file_name().to_string_lossy().to_string())),
                                      Context { is_static: true, static_response: Some(Response {protocolo: "HTTP/1.1".to_string(), status: "200 OK".to_string(),
                                                  header: HashMap::from([get_content_type(path.path().extension().and_then(OsStr::to_str)
                                                                                          .unwrap_or("")).expect("File has no extension")]),
                                                  body: read_file(File::open(path.path().to_string_lossy().to_string()).unwrap())}), handler: None });
      }
    } else {
      for result_path in directory {
        let path = result_path.unwrap();
        self.contexts.insert((Method::GET, format!("/{}/{}", at, path.file_name().to_string_lossy().to_string())), 
                                      Context { is_static: true, static_response: Some(Response {protocolo: "HTTP/1.1".to_string(), status: "200 OK".to_string(),
                                                  header: HashMap::from([get_content_type(path.path().extension().and_then(OsStr::to_str)
                                                                                          .unwrap_or("")).expect("File has no extension")]),
                                                  body: read_file(File::open(path.path().to_string_lossy().to_string()).unwrap())}), handler: None });
      }
    }
  }

  fn read_request(&self, mut stream: &TcpStream) -> Request {
    let mut buf = [0u8; 4096];
    match stream.read(&mut buf) {
      Ok(_) => {
        Request::from(String::from_utf8_lossy(&buf).trim())
      },
      Err(e) => {
        eprintln!("Failed to read from stream. Err: {}", e);
        return Request::default();
      }
    }
  }
  fn send_response(&mut self, mut stream: &TcpStream, request: &Request) -> Response {
    let mut response = Response { protocolo: "HTTP/1.1".to_string(), status: "404 NOT FOUND".to_string(), header: HashMap::from([("Content-Type".to_string(), "text/html; charset=UTF-8".to_string())]),
                                           body: "<html>\r\n<body>\r\n\t<h1>404</h1>\r\n\t<p>Page Not Found</p>\r\n</body>\r\n</html>".to_string() };
    for context in &self.contexts {
      if context.0.1 == request.uri.path {
        if context.1.is_static && request.method == Method::GET {
          response = context.1.static_response.clone().expect("Context is static but static_response field is None")
        }
        else if context.0.0 == request.method {
          response = (context.1.handler.expect("Context is handled but handler field is None"))(request);
          break
        }
      }
    }
    stream.write_all(response.to_string().as_bytes()).expect("Send response interrupted");
    response
  }

  pub fn listen(&self, port: u16) {
    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port))).expect(&format!("Failed to bind to 127.0.0.1:{}", port));
    for stream in listener.incoming() {
      match stream {
        Ok(stream) => { let mut self_clone = self.clone();
          thread::spawn(move || {
            let request = self_clone.read_request(&stream);
            let response = self_clone.send_response(&stream, &request);
            println!("Request:\n{}\n\nResponse:\n{}\n\n", request.to_string(), response.to_string());
          });
        },
        Err(e) => { println!("Error: {}", e); }
      }
    }
  }
}