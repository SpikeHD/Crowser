use std::str::FromStr;

use include_dir::Dir;
use tiny_http::{Header, Response, Server};

use crate::error::CrowserError;

pub enum WebserverMessage {
  Kill,
}

pub struct Webserver {
  server: Server,
  directory: Dir<'static>,
}

impl Webserver {
  pub fn new(port: u16, directory: Dir<'static>) -> Result<Self, CrowserError> {
    let server = match Server::http(format!("127.0.0.1:{}", port)) {
      Ok(server) => server,
      Err(err) => return Err(CrowserError::WebserverError(format!("Failed to bind to port {}: {}", port, err))),
    };

    Ok(Self {
      server,
      directory,
    })
  }

  pub fn poll_request(&self) {
    if let Ok(Some(request)) = self.server.try_recv() {
      let mut path = request.url().strip_prefix('/').unwrap_or(request.url());

      // If the path is empty, we should serve the index.html file
      if path.is_empty() {
        path = "index.html";
      }

      let file = self.directory.get_file(path);

      if file.is_none() {
        request.respond(
          Response::empty(404)
        ).unwrap_or_default();
        return;
      }

      let file = file.unwrap();
      let contents = file.contents_utf8().unwrap();
      let mime = mime_guess::from_path(file.path()).first_or_octet_stream();
      let mut res = Response::from_string(contents);

      // Headers
      let content_type = Header::from_str(format!("Content-Type: {}", mime).as_str()).unwrap();
      let content_length =
        Header::from_str(format!("Content-Length: {}", contents.len()).as_str()).unwrap();

      res.add_header(content_type);
      res.add_header(content_length);

      request.respond(res).unwrap_or_default();
    }
  }
}
