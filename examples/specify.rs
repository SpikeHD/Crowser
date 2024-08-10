use crowser::{browser::BrowserKind, Window};

fn main() {
  let mut profile_dir = std::env::current_dir().unwrap();
  profile_dir.push("example_profiles");

  // Specify Firefox-based browsers
  let mut win = Window::new(Some(BrowserKind::Gecko), profile_dir).unwrap();
  win.set_url("https://example.com/");

  win.clear_profile().unwrap();

  win.create().unwrap();
}