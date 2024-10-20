use crowser::{error::CrowserError, RemoteConfig, Window};

fn main() -> Result<(), CrowserError> {
  let mut profile_dir = std::env::current_dir()?;
  profile_dir.push("example_profiles");

  let config = RemoteConfig {
    url: "https://example.com".to_string(),
  };

  let mut window = Window::new(config, None, profile_dir)?;
  let ipc = window.ipc();

  window.clear_profile().unwrap_or_default();

  std::thread::spawn(move || {
    ipc.block_until_initialized().unwrap_or_default();

    let two = ipc.eval("1 + 1").unwrap();
    println!("1 + 1 = {:?}", two);

    std::thread::sleep(std::time::Duration::from_secs(2));

    ipc.eval("alert('Hello from Crowser!')").unwrap_or_default();
  });

  window.create()?;

  Ok(())
}
