use crowser::{error::CrowserError, RemoteConfig, Window};

fn main() -> Result<(), CrowserError> {
  let mut profile_dir = std::env::current_dir()?;
  profile_dir.push("example_profiles");

  let config = RemoteConfig {
    url: "https://example.com".to_string(),
  };

  let mut window = Window::new(config, None, profile_dir)?;

  match window.clear_profile() {
    Ok(_) => {}
    Err(err) => {
      println!("Error clearing profile: {}", err);
    }
  };

  window.create()?;

  // Clear once the window is closed
  window.clear_profile().unwrap_or_default();

  Ok(())
}
