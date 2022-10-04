use http_server::*;
use std::{collections::HashMap, fs::read_dir, thread, time};

fn main() {
  let mut server = HttpServer::new();
  
  server.add_static_files(".\\public\\", "");

  server.listen(3000);
}