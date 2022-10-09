use http_server::*;
use std::thread;
use std::collections::HashMap;

fn main() {
  let server = HttpServer::new();
  server.listen(8080, |stream, mut server| {
    server.add_static_files("./public/", "");

    thread::spawn(move || {
      let request = server.read_request(&stream);
      let response = server.send_response(&stream, &request, HashMap::new());
      println!("Request:\n{}\nResponse:\n{}", request.to_string(), response.to_string());
    });
  })
}