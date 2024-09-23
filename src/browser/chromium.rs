use std::path::PathBuf;

use crate::{ContentConfig, Window};

/// In order to prevent profile collisions, the main user-provided profile directory is supplemented with additional folders.
pub fn get_profile_dir(win: &Window) -> PathBuf {
  let mut profile_dir = win.profile_directory.clone();
  profile_dir.push("chrome");
  profile_dir.push("profile");

  profile_dir
}

/// Generate command line options required to make Chromium-based browsers
/// look like a standalone app.
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
    match &win.config {
      ContentConfig::Remote(config) => {
        format!("--app={}", config.url)
      },
      ContentConfig::Local(config) => {
        format!("--app=http://localhost:{}", config.port)
      },
    },

    // Profile
    if let Some(profile) = get_profile_dir(win).to_str() {
      format!("--user-data-dir={}", profile)
    } else {
      "".to_string()
    },
  ]);

  if win.disable_hardware_acceleration {
    options.push("--disable-gpu".to_string());
  }

  if let Some(config) = &win.chromium_config {
    for ext in &config.extensions {
      options.push(format!("--load-extension={}", ext.display()));
    }
  }

  options
}

pub fn write_extra_profile_files(_win: &Window) -> Result<(), std::io::Error> {
  Ok(())
}
