use crowser::Window;

fn main() {
  let mut profile_dir = std::env::current_dir().unwrap();
  profile_dir.push("example_profiles");

  let mut window = Window::new(None, profile_dir).unwrap();
  window.set_url("https://example.com/");

  window.clear_profile().unwrap();

  window.create().unwrap();
}