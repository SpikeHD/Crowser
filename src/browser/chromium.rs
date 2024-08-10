use crate::Window;

/// Generate command line options required to make Chromium-based browsers
/// look like a standalone app.
/// 
/// Things like the initial URL, window size, etc. are available on the `Window`
pub fn generate_cli_options(win: &Window) -> Vec<String> {
  let mut options = vec![];

  // Basic
  options.push(format!("--app={}", win.url));

  // Profile directory
  options.push(format!("--user-data-dir={}", win.profile_directory.to_str().unwrap()));

  // Features to make it work normal
  options.push("--disable-features=WinRetrieveSuggestionsOnlyOnDemand,HardwareMediaKeyHandling,MediaSessionService".to_string());

  if win.disable_hardware_acceleration {
    options.push("--disable-gpu".to_string());
  }

  options
}