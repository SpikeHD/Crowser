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

pub fn write_extra_profile_files(win: &Window) -> Result<(), std::io::Error> {
  let mut prefs = win.profile_directory.clone();
  prefs.push("user.js");

  // Create profile folder
  std::fs::create_dir_all(&win.profile_directory)?;
  
  let pref_str = format!(
    r#"
user_pref("browser.startup.homepage", "about:blank");
user_pref("browser.startup.page", 0);

// Disable first run stuff
user_pref("browser.shell.checkDefaultBrowser", false);
user_pref("browser.rights.3.shown", true);
user_pref('toolkit.telemetry.reportingpolicy.firstRun', false);

// Window size
user_pref('privacy.window.maxInnerWidth', {});
user_pref('privacy.window.maxInnerHeight', {});

// Hardware acceleration
user_pref('gfx.webrender.all', {});

// For IPC
user_pref('devtools.chrome.enabled', true);
user_pref('devtools.debugger.prompt-connection', false);
user_pref('devtools.debugger.remote-enabled', true);

// Media (ie autoplay)
user_pref('media.autoplay.blocking_policy', false);

user_pref("toolkit.legacyUserProfileCustomizations.stylesheets", true);
    "#,
    win.width,
    win.height,
    !win.disable_hardware_acceleration
  );

  std::fs::write(prefs, pref_str)?;

  Ok(())
}