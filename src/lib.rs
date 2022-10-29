use std::{collections::HashMap, fs::{File, read_dir}, io::*,
          net::{TcpListener, TcpStream, SocketAddr}, ffi::OsStr, hash::Hash};

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
  let mut headers: HashMap<String, String> = HashMap::new();

  for str in (&s[s.find("\r\n").unwrap_or(s.len())+1..s.find("\r\n\r\n").unwrap_or(s.len())]).split("\r\n") {
    let vec_str = str.split(": ").collect::<Vec<&str>>();
    if vec_str.len() > 1 {
      match vec_str[0].rfind("\n") {
        Some(key_start) => { headers.insert(vec_str[0][key_start+1..].to_string(), vec_str[1].to_string()); }
        None => { headers.insert(vec_str[0].to_string(), vec_str[1].to_string()); }
      }
    }
  }

  headers
}

pub fn get_content_type(file_ext: &str) -> (String, String) {
  match file_ext.to_lowercase().as_str() {
    "html" => ("Content-Type".to_string(), "text/html; charset=UTF-8".to_string()),
    "css" => ("Content-Type".to_string(), "text/css".to_string()),
    "js" => ("Content-Type".to_string(), "text/javascript".to_string()),
    "png" => ("Content-Type".to_string(), "image/png".to_string()),
    "jpg" => ("Content-Type".to_string(), "image/jpeg".to_string()),
    "jpeg" => ("Content-Type".to_string(), "image/jpeg".to_string()),
    "gif" => ("Content-Type".to_string(), "image/gif".to_string()),
    "ico" => ("Content-Type".to_string(), "image/x-icon".to_string()),
    "svg" => ("Content-Type".to_string(), "image/svg+xml".to_string()),
    "mid" => ("Content-Type".to_string(), "audio/midi".to_string()),
    "mp3" => ("Content-Type".to_string(), "audio/mpeg".to_string()),
    "wav" => ("Content-Type".to_string(), "audio/wav".to_string()),
    "mp4" => ("Content-Type".to_string(), "video/mp4".to_string()),
    "json" => ("Content-Type".to_string(), "application/json".to_string()),
    _ => ("Content-Type".to_string(), "text/plain".to_string())
  }
}

pub fn read_file(path: &str) -> Result<Vec<u8>> {
  let mut contents = Vec::new();
  
  match File::open(path) {
    Ok(mut file) => {
      match file.read_to_end(&mut contents) {
        Ok(_) => Ok(contents),
        Err(e) => return Err(e)
      }
    },
    Err(e) => return Err(e)
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
    match self.query {
      Some(_) => format!("{}?{}", self.path, header_to_string(self.query.clone().unwrap())),
      None => self.path.clone()
    }
  }
}
impl From<&str> for Uri {
  fn from(s: &str) -> Self {
    if let Some(query_start) = s.find("?") {
      let mut query: HashMap<String, String> = HashMap::new();

      for param in (&s[query_start+1..]).split("&") {
        let mut key_value_splitted = param.split("=");

        let k = key_value_splitted.next().expect("query has no key").to_string();
        let v = key_value_splitted.next().expect(&format!("{} has no value", k)).to_string();

        query.insert(k, v);
      }
      return Uri {
        path: (&s[..query_start]).to_string(),
        query: Some(query)
      } 
    }

    Uri {
      path: s.to_string(),
      query: None
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
  DELETE,
}
impl ToString for Method {
  fn to_string(&self) -> String {
    match *self {
      Method::GET => format!("GET"),
      Method::POST => format!("POST"),
      Method::DELETE => format!("DELETE"),
    }
  }
}
impl From<&str> for Method {
  fn from(s: &str) -> Self {
    match s {
      "GET" => Method::GET,
      "POST" => Method::POST,
      "DELETE" => Method::DELETE,
      _ => panic!("inexistent method")
    }
  }
}

pub struct Request {
  pub method: Method,
  pub uri: Uri,
  pub protocolo: String,
  pub header: HashMap<String, String>,
  pub body: Option<String>
}
impl ToString for Request {
  fn to_string(&self) -> String {
    format!("{} {} {}\r\n{}\r\n{}", self.method.to_string(), self.uri.to_string(), self.protocolo, header_to_string(self.header.clone()),
                                    self.body.clone().unwrap_or(String::new()))
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
    let mut vec_status_line = (&s[..s.find("\r\n").expect("Request has no header")]).split(" "); 
    let header: HashMap<String, String> = header_from_string(s);

    let mut body: Option<String> = None;
    if s.find("\r\n\r\n").is_some() {
      body = Some((&s[s.find("\r\n\r\n").unwrap()+4..]).to_string());
    }

    Request { method: Method::from(vec_status_line.next().expect("request has no method")), uri: Uri::from(vec_status_line.next().expect("request has no URI")),
              protocolo: vec_status_line.next().expect("request has no protocolo").to_string(), header: header, body: body }
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
  pub body: Vec<u8>
}
impl Response {
  fn as_bytes(&self) -> Vec<u8> {
    let mut out = vec![];
    out.append(&mut self.protocolo.as_bytes().to_vec());
    out.append(&mut self.status.as_bytes().to_vec());
    out.append(&mut header_to_string(self.header.clone()).as_bytes().to_vec());
    out.append(&mut self.body.clone());
    out
  }
}
impl ToString for Response {
  fn to_string(&self) -> String {
    format!("{} {}\r\n{}\r\n{}", self.protocolo, self.status,
            header_to_string(self.header.clone()),
            String::from_utf8(self.body.clone()).unwrap_or(String::new()))
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
    Response { protocolo: (&s[..s.find(" ").expect("Response has no protocolo")]).to_string(),
               status: (&s[s.find(" ").unwrap()+1..s.find("\r\n").unwrap_or(s.find("\r\n\r\n").expect("Response has an empty body"))]).to_string(),
               header: header_from_string(s), body: (&s[s.find("\r\n\r\n").unwrap()..]).as_bytes().to_vec() }
  }
}
impl Default for Response {
  fn default() -> Self {
    Response { protocolo: "HTTP/1.1".to_string(), status: "200 OK".to_string(), header: HashMap::new(), body: vec![] }
  }
}

pub struct HttpServer {
  contexts: HashMap<(Method, String), Response>
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

  pub fn add_static_files(&mut self, dir_path: &str, opt_at: Option<&str>) {
    let directory = match read_dir(&dir_path) {
      Ok(d) => d,
      Err(e) => {
        eprintln!("Problem reading directory: {}\r\nErr: {}", dir_path, e);
        return
      }
    };
    if let Some(at) = opt_at {
      for result_path in directory {
        let path = result_path.unwrap();
        if path.metadata().expect("cannot get metadata").is_dir() { self.add_static_files(&path.path().to_string_lossy().to_string(), Some(at)); }
        else {
          match read_file(&path.path().to_string_lossy().to_string()) {
            	Ok(content) => {
                self.contexts.insert((Method::GET, format!("/{}/{}", at.replace("\\", "/"), path.file_name().to_string_lossy().to_string().replace("\\", "/"))), 
                                      Response {protocolo: "HTTP/1.1".to_string(), status: "200 OK".to_string(),
                                                  header: HashMap::from([get_content_type(path.path().extension().and_then(OsStr::to_str)
                                                                              .unwrap_or(""))]),
                                                  body: content.clone()});
                if path.file_name().to_string_lossy().to_string().eq("index.html") {
                  self.contexts.insert((Method::GET, format!("/{}", at.replace("\\", "/"))),
                                      Response {protocolo: "HTTP/1.1".to_string(), status: "200 OK".to_string(),
                                                  header: HashMap::from([get_content_type(path.path().extension().and_then(OsStr::to_str).unwrap_or(""))]),
                                                  body: content});
                }
              } ,
              Err(_) => { eprintln!("can't read {}", path.path().to_string_lossy().to_string()); }
          }
        }
      }
    }
    else {
      for result_path in directory {
        let path = result_path.unwrap();
      
        if path.metadata().expect("cannot get metadata").is_dir() { self.add_static_files(&path.path().to_string_lossy().to_string(), Some(&path.path().to_string_lossy().to_string()[path.path().to_string_lossy().find("\\").unwrap()+1..])); }
        else {
          match read_file(&path.path().to_string_lossy().to_string()) {
            Ok(content) => {
              println!("/{}", path.file_name().to_string_lossy().to_string());
              self.contexts.insert((Method::GET, format!("/{}", path.file_name().to_string_lossy().to_string())),
                                  Response {protocolo: "HTTP/1.1".to_string(), status: "200 OK".to_string(),
                                              header: HashMap::from([get_content_type(path.path().extension().and_then(OsStr::to_str).unwrap_or(""))]),
                                              body: content.clone()});
              if path.file_name().to_string_lossy().to_string().eq("index.html") {
                self.contexts.insert((Method::GET, "/".to_string()),
                                    Response {protocolo: "HTTP/1.1".to_string(), status: "200 OK".to_string(),
                                                header: HashMap::from([get_content_type(path.path().extension().and_then(OsStr::to_str).unwrap_or(""))]),
                                                body: content});
              }
            },
            Err(_) => eprintln!("can't read {}", path.path().to_string_lossy().to_string())
          }
        }
      }
    }
  }

  pub fn read_request(mut stream: &TcpStream) -> Request {
    let mut buf = [0u8; 4096];
    match stream.read(&mut buf) {
      Ok(_) => {
        Request::from(String::from_utf8_lossy(&buf).trim())
      },
      Err(e) => {
        eprintln!("Failed to read from stream. Err: {}", e);
        Request::default()
      }
    }
  }
  pub fn send_response(self, mut stream: &TcpStream, request: &Request,
                        context_handlers: HashMap<(Method, String), Box<&(dyn Fn(&Request) -> Response + Sync)>>) -> Response {
    if let Some(response) = self.contexts.get(&(request.method, request.uri.path.clone())) {
      stream.write_all(&response.as_bytes()).expect("Send response interrupted");
        return response.clone();
    }
    if let Some(handler) = context_handlers.get(&(request.method, request.uri.path.clone())) {
      let response = (handler)(request);
      stream.write_all(&response.as_bytes()).expect("Send response interrupted");
      return response;
    }

    let not_found = Response {protocolo: "HTTP/1.1".to_string(), status: "404 NOT FOUND".to_string(), header: HashMap::from([("Content-Type".to_string().to_string(), "text/html; charset=UTF-8".to_string())]),
                                body: "<html>\r\n<body>\r\n\t<h1>404</h1>\r\n\t<p>Page Not Found</p>\r\n</body>\r\n</html>".as_bytes().to_vec() };
    stream.write_all(&not_found.as_bytes()).expect("Send response interrupted");
    not_found
  }

  pub fn listen<F, F2>(self, port: u16, handler: F, on_bind: F2) where F: Fn(TcpStream, HttpServer),
                                                                       F2: Fn() {
    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port))).expect(&format!("Failed to bind to 127.0.0.1:{}", port));
    on_bind();

    for stream in listener.incoming() {
      match stream {
        Ok(stream) => { handler(stream, self.clone()); },
        Err(e) => { println!("Error: {}", e); }
      }
    }
  }
}