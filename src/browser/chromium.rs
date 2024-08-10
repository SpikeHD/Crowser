use crate::Window;

/// Generate command line options required to make Chromium-based browsers
/// look like a standalone app.
/// 
/// Things like the initial URL, window size, etc. are available on the `Window`
pub fn generate_cli_options(win: &Window) -> Vec<String> {
  let mut options = vec![];

  // Basic
  options.extend([
    "--disable-translate".to_string(),
    "--disable-popup-blocking".to_string(),
    "--disable-sync".to_string(),
    "--no-first-run".to_string(),
    "--no-default-browser-check".to_string(),
    "--disable-features=AutofillServerCommunication,WinRetrieveSuggestionsOnlyOnDemand,MediaSessionService,HardwareMediaKeyHandling".to_string(),
    // Configurable stuff
    format!("--window-size={},{}", win.width, win.height),
    format!("--app={}", win.url),

    // Profile
    format!("--user-data-dir={}", win.profile_directory.to_str().unwrap())
  ]);

  if win.disable_hardware_acceleration {
    options.push("--disable-gpu".to_string());
  }

  options
}