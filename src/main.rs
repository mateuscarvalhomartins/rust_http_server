use http_server::*;
use std::thread;
use std::collections::HashMap;
use std::time::Instant;

fn main() {
  let mut server = HttpServer::new();
  
  server.add_static_files("public", None);

  server.listen(8080, |stream, server| {
    thread::spawn(move || {
      let now = Instant::now();

      let request = HttpServer::read_request(&stream);
      let response = server.send_response(&stream, &request, HashMap::new());

      let elapsed = now.elapsed();
      
      println!("Request:\n{}\nResponse:\n{}", request.to_string(), response.to_string());
      println!("Elapsed: {:.2?}", elapsed);
    });
  }, || { println!("Server running at http://locahost:8080/") })
}