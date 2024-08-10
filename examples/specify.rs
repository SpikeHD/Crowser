use crowser::{browser::BrowserKind, error::CrowserError, RemoteConfig, Window};

fn main() -> Result<(), CrowserError> {
  let mut profile_dir = std::env::current_dir()?;
  profile_dir.push("example_profiles");

  let config = RemoteConfig {
    url: "https://example.com".to_string(),
  };

  // Specify Firefox-based browsers
  let mut win = Window::new(config, Some(BrowserKind::Gecko), profile_dir)?;

  win.clear_profile()?;

  win.create()?;

  Ok(())
}
