use crowser::{RemoteConfig, Window};

fn main() {
  let mut profile_dir = std::env::current_dir().unwrap();
  profile_dir.push("example_profiles");

  let config = RemoteConfig {
    url: "https://example.com".to_string(),
  };

  let mut window = Window::new(config, None, profile_dir).unwrap();

  window.clear_profile().unwrap();

  window.create().unwrap();
}