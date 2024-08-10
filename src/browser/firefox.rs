use crate::Window;

/// Generate command line options required to make Firefox-based browsers
/// look like a standalone app.
/// 
/// Things like the initial URL, window size, etc. are available on the `Window`
pub fn generate_cli_options(win: &Window) -> Vec<String> {
  let mut options = vec![];

  // Basic
  options.push("--new-window".to_string());

  options.push("--url".to_string());
  options.push(win.url.clone());

  // Profile directory
  options.push("--profile".to_string());
  options.push(win.profile_directory.to_str().unwrap().to_string());
  
  options
}