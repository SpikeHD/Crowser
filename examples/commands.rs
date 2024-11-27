use crowser::{error::CrowserError, RemoteConfig, Window};

fn main() -> Result<(), CrowserError> {
  let mut profile_dir = std::env::current_dir()?;
  profile_dir.push("example_profiles");

  let config = RemoteConfig {
    url: "https://example.com".to_string(),
  };

  let mut window = Window::new(config, None, profile_dir)?;
  let ipc = window.ipc();

  let ipc_thread = std::thread::spawn(move || setup_commands(&ipc));

  window.clear_profile().unwrap_or_default();

  std::thread::spawn(move || {

  });

  window.create()?;

  ipc_thread.join().expect("Failed to join IPC thread").expect("IPC thread panicked");

  Ok(())
}

fn setup_commands(ipc: &crowser::WindowIpc) -> Result<(), CrowserError> {
  ipc.block_until_initialized()?;

  ipc
    .register_command("hello", |_| {
      println!("Got hello command");
      Ok(serde_json::json!("Hello from Crowser!"))
    })?;

  std::thread::sleep(std::time::Duration::from_secs(1));

  println!("Waiting for result...");
  // Eval some JS that calls that command
  let result = ipc
    .eval("window.__CROWSER.ipc.invoke('hello')")?;
  println!("Result: {:?}", result);

  Ok(())
}