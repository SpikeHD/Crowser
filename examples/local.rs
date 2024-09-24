use crowser::{error::CrowserError, include_dir, LocalConfig, Window};

fn main() -> Result<(), CrowserError> {
  let mut profile_dir = std::env::current_dir()?;
  profile_dir.push("example_profiles");

  let dir = include_dir::include_dir!("$CARGO_MANIFEST_DIR/examples/local");

  //  To be safe, we'll try to find an open port between 9000 and 9999
  for port in 9000..9999 {
    let config = LocalConfig {
      port,
      directory: dir.clone(),
    };

    let mut window = Window::new(config, None, profile_dir.clone())?;

    match window.clear_profile() {
      Ok(_) => {}
      Err(err) => {
        println!("Error clearing profile: {}", err);
      }
    };

    match window.create() {
      Ok(_) => {
        println!("Window created on port {}", port);

        // Once the window is closed, the profile will be cleared.
        window.clear_profile().unwrap_or_default();
        break;
      }
      Err(e) => {
        println!("Error creating window on port {}: {:?}", port, e);
      }
    }
  }

  Ok(())
}
