use std::{fs, path::PathBuf};

use crate::Window;

pub fn write_profile(app_name: String, profile_directory: PathBuf, initialization_script: String) -> Result<(), std::io::Error> {
  if !profile_directory.exists() {
    fs::create_dir_all(&profile_directory)?;
  }

  Ok(())
}

/// Generate command line options required to make Firefox-based browsers
/// look like a standalone app.
/// 
/// Things like the initial URL, window size, etc. are available on the `Window`
pub fn generate_cli_options(win: &Window) -> Vec<String> {
  let mut options = vec![];

  options.push("--new-window".to_string());
  options.push("--url".to_string());
  options.push(win.url.clone());
  
  options
}