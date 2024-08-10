use std::path::PathBuf;

use crowser::{LocalConfig, RemoteConfig, Window};
use include_dir::include_dir;

fn main() {
  let mut profile_dir = std::env::current_dir().unwrap();
  profile_dir.push("example_profiles");

  let dir = include_dir!("$CARGO_MANIFEST_DIR/examples/local");

  for port in 8000..8050 {
    let config = LocalConfig {
      port: port,
      directory: dir.clone(),
    };

    let mut window = Window::new(config, None, profile_dir.clone()).unwrap();

    window.clear_profile().unwrap();

    match window.create() {
      Ok(_) => {
        println!("Window created on port {}", port);
        break;
      }
      Err(e) => {
        println!("Error creating window on port {}: {:?}", port, e);
      }
    }
  }
}
