use crowser::{error::CrowserError, ipc::BrowserIpc, RemoteConfig, Window};

fn main() -> Result<(), CrowserError> {
  let mut profile_dir = std::env::current_dir()?;
  profile_dir.push("example_profiles");

  let config = RemoteConfig {
    url: "https://example.com".to_string(),
  };

  let mut window = Window::new(config, None, profile_dir)?;
  let root_ipc = window.get_ipc();

  window.clear_profile().unwrap_or_default();

  std::thread::spawn(move || {
    let mut ipc: BrowserIpc;

    // Wait for IPC to be initialized
    loop {
      std::thread::sleep(std::time::Duration::from_millis(10));

      let mut root_ipc = match root_ipc.try_lock() {
        Ok(val) => val,
        Err(_) => continue,
      };

      if let Some(root_ipc) = root_ipc.as_mut() {
        ipc = root_ipc.clone();
        break;
      }
    }

    ipc
      .register_command(
        "hello",
        Box::new(|_| {
          println!("Got hello command");
          Ok(serde_json::json!("Hello from Crowser!"))
        }),
      )
      .unwrap_or_default();

    std::thread::sleep(std::time::Duration::from_secs(1));

    // Eval some JS that calls that command
    let result = ipc.eval("window.__CROWSER.ipc.invoke('hello')").unwrap_or_default();
    println!("Result: {:?}", result);
  });

  window.create()?;

  Ok(())
}
