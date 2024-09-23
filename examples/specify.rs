use crowser::{browser::BrowserKind, error::CrowserError, RemoteConfig, Window};

fn main() -> Result<(), CrowserError> {
  let mut profile_dir = std::env::current_dir()?;
  profile_dir.push("example_profiles");

  let config = RemoteConfig {
    url: "https://example.com".to_string(),
  };

  // Specify Firefox-based browsers
  let mut win = Window::new(config, Some(BrowserKind::Gecko), profile_dir)?;

  match win.clear_profile() {
    Ok(_) => {}
    Err(err) => {
      println!("Error clearing profile: {}", err);
    }
  };

  win.create()?;

  // Clear once the window is closed
  win.clear_profile().unwrap_or_default();

  Ok(())
}
