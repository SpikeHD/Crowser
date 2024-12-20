use crowser::{error::CrowserError, RemoteConfig, Window};

fn main() -> Result<(), CrowserError> {
  let mut profile_dir = std::env::current_dir()?;
  profile_dir.push("example_profiles");

  let config = RemoteConfig {
    url: "https://example.com".to_string(),
  };

  let mut window = Window::new(config, None, profile_dir)?;
  window.set_initialization_script("window.alert('Hello from Crowser!')")?;

  window.clear_profile().unwrap_or_default();

  window.create()?;

  Ok(())
}
