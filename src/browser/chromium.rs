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

  options
}