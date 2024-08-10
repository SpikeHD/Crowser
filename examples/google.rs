use crowser::Window;

fn main() {
  let profile_dir = std::env::current_dir().unwrap();
  profile_dir.join("example_profiles").to_str().unwrap().to_string();

  let mut window = Window::new("Google Example".to_string(), None, profile_dir).unwrap();
  window.set_url("https://google.com");

  window.create().unwrap();
}