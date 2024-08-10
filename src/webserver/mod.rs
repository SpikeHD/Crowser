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
      let response = Response::from_string("Hello, world!".to_string())
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/plain"[..]).unwrap());
      request.respond(response).unwrap();
    }
  }
}
