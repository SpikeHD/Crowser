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
    ipc
      .register_command(
        "hello",
        |_| {
          println!("Got hello command");
          Ok(serde_json::json!("Hello from Crowser!"))
        },
      )
      .unwrap_or_default();

    std::thread::sleep(std::time::Duration::from_secs(1));

    // Eval some JS that calls that command
    let result = ipc
      .eval("window.__CROWSER.ipc.invoke('hello')")
      .unwrap_or_default();
    println!("Result: {:?}", result);
  });

  window.create()?;

  Ok(())
}
