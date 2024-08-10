use crate::{ContentConfig, Window};

/// Generate command line options required to make Firefox-based browsers
/// look like a standalone app.
/// 
/// Things like the initial URL, window size, etc. are available on the `Window`
pub fn generate_cli_options(win: &Window) -> Vec<String> {
  let mut options = vec![];

  // Basic
  options.push("--new-window".to_string());

  options.push("--url".to_string());

  match &win.config {
    ContentConfig::Remote(config) => {
      options.push(config.url.clone());
    },
    ContentConfig::Local(config) => {
      options.push(format!("http://localhost:{}", config.port.unwrap()));
    }
  }

  // Profile directory
  options.push("--profile".to_string());
  options.push(win.profile_directory.to_str().unwrap().to_string());
  
  options
}

pub fn write_extra_profile_files(win: &Window) -> Result<(), std::io::Error> {
  let mut prefs = win.profile_directory.clone();
  prefs.push("user.js");

  let mut user_css = win.profile_directory.clone();
  user_css.push("chrome");

  // Create profile folder
  std::fs::create_dir_all(&win.profile_directory)?;
  std::fs::create_dir_all(&user_css)?;

  user_css.push("userChrome.css");
  
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
user_pref('layers.acceleration.force-enabled', {});

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
    !win.disable_hardware_acceleration,
    !win.disable_hardware_acceleration
  );

  std::fs::write(prefs, pref_str)?;

  let mut css_str = format!(
    r#"
    /* Disable the entire URL bar */
    #urlbar-container, #nav-bar, #TabsToolbar-customization-target, .notificationbox-stack {{
      visibility: collapse;
    }}
    "#
  );

  if let Some(config) = &win.firefox_config {
    css_str.push_str(config.custom_css.as_ref().unwrap_or(&String::new()));
  }

  std::fs::write(user_css, css_str)?;

  Ok(())
}