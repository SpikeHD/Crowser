use crowser::{include_dir, LocalConfig, Window, error::CrowserError};

fn main() -> Result<(), CrowserError> {
  let mut profile_dir = std::env::current_dir()?;
  profile_dir.push("example_profiles");

  let dir = include_dir::include_dir!("$CARGO_MANIFEST_DIR/examples/local");

  for port in 8000..8050 {
    let config = LocalConfig {
      port,
      directory: dir.clone(),
    };

    let mut window = Window::new(config, None, profile_dir.clone())?;

    window.clear_profile()?;

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

  Ok(())
}
