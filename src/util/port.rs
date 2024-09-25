use std::net::TcpListener;

// This is a meh solution but its way better than a static port lol
pub fn port_is_available(port: u16) -> bool {
  TcpListener::bind(("127.0.0.1", port)).is_ok()
}

pub fn get_available_port(start_at: Option<u16>) -> u16 {
  let mut port = start_at.unwrap_or(8000);

  while !port_is_available(port) {
    port += 1;
  }

  port
}