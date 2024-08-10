use crowser::{browser::BrowserKind, RemoteConfig, Window};

fn main() {
  let mut profile_dir = std::env::current_dir().unwrap();
  profile_dir.push("example_profiles");

  let config = RemoteConfig {
    url: "https://example.com".to_string(),
  };

  // Specify Firefox-based browsers
  let mut win = Window::new(config, Some(BrowserKind::Gecko), profile_dir).unwrap();

  win.clear_profile().unwrap();

  win.create().unwrap();
}
