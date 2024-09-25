/*! # Crowser

Create "desktop apps" using user-installed browsers.

## Main features
* ~1MB binary size with minimal dependencies (except for the browser, of course!)
* Multi-platform. Supports Windows, macOS, and Linux.
* Support for both local and remote websites.
* Maximizes performance of whatever browser is chosen, and uses an entirely separate browser profile.

**NOTE**: This library will basically forever and always be intended for low-stakes or experimental use. It's really hard to guarantee working functionality and consistency across a bunch of different browsers, so please keep that in mind!

## Browser support

All browser support comes with a few small caveats. You may notice small inconsistencies between, say, your app running in Firefox and in Chrome. Many of these are also not properly tested, so if you find any issues, feel free to [submit an issue](https://github.com/SpikeHD/Crowser/issues/choose)!

* Chrome/Chromium (stable, beta, dev, canary)
* Edge (stable, beta, dev, canary)
* Firefox (stable, beta, dev, nightly)
* Floorp
* Thorium
* Brave
* Vivaldi
* Librewolf
* Waterfox
* Mercury

# Usage

More examples can be found in the [examples](./examples) directory. Try them with `cargo run --example <example>`!

## Displaying a remote website
```rust
use crowser::{error::CrowserError, RemoteConfig, Window};

fn main() -> Result<(), CrowserError> {
  // Profile directories are specified by you, so put it wherever makes sense!
  let mut profile_dir = PathBuf::from("/path/to/your/app/profiles");

  let config = RemoteConfig {
    url: "https://example.com".to_string(),
  };

  let mut window = Window::new(config, None, profile_dir)?;

  // Make sure the profile is brand-new before launch
  window.clean_profile()?;

  // This will spawn the window and block until it is closed
  window.create()?;

  Ok(())
}
```

## Embedding a local website
```rust
use crowser::{error::CrowserError, include_dir, LocalConfig, Window};

fn main() -> Result<(), CrowserError> {
  let mut profile_dir = PathBuf::from("/path/to/your/app/profiles");

  // include_dir is a re-export of the include_dir crate
  let dir = include_dir::include_dir!("/path/to/your/app/dist");
  let config = LocalConfig {
    port,
    directory: dir.clone(),
  };

  let mut window = Window::new(config, None, profile_dir.clone())?;

  window.clean_profile()?;

  // Since we're looping, we'll break when we successfully create the window. This will
  // actually block the thread until the window is closed.
  match window.create() {
    Ok(_) => {
      println!("Window created on port {}", port);
      break;
    }
    Err(e) => {
      println!("Error creating window on port {}: {:?}", port, e);
    }
  }

  Ok(())
}
```

# How does it work?

On a high level, Crowser works by first detecting browser installations on the user's system (using known paths and ~~registry keys~~). Then, depending on the browser chosen, it will make some specific changes to the browser's CLI arguments,
profile directory, or both. For example, for Firefox there is a `user.js` file in all profiles that can control much of the browser's default behavior. In Chromium-based browsers, there are a stupid amount of command-line arguments that can be
used to control the browser's behavior ([check out this huge list!](https://peter.sh/experiments/chromium-command-line-switches/)).

IPC is facilitated through the [Chrome DevTools Protocol](https://chromedevtools.github.io/devtools-protocol/). To keep the binary size small, the implementation is custom and therefore a little scuffed, but developers do not have to
care about it anyways!
*/

use std::{
  path::PathBuf,
  sync::{atomic::AtomicBool, Arc, Mutex},
};

use browser::{get_browser_path, Browser, BrowserKind};
use error::CrowserError;
use include_dir::Dir;
use shared_child::SharedChild;

pub mod browser;
mod cdp;
pub mod error;
pub mod ipc;
mod util;
mod webserver;

// Re-export the include_dir macro
pub use include_dir;
use util::port::get_available_port;
use webserver::{Webserver, WebserverMessage};

/// Firefox/Gecko-specific configuration options. These have no effect if the window is not a Firefox window.
#[derive(Debug)]
pub struct FirefoxConfig {
  /// Custom Firefox CSS to apply to the window (NOT the page!). See https://www.userchrome.org/ for more details
  pub custom_css: Option<String>,
}

/// Chromium-specific configuration options. These have no effect if the window is not a Chromium window.
#[derive(Debug)]
pub struct ChromiumConfig {
  /// List of paths to unpacked extensions to load into the browser.
  pub extensions: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub enum ContentConfig {
  Local(LocalConfig),
  Remote(RemoteConfig),
}

/// Configuration for a local (i.e bundled) website/web app
#[derive(Debug, Clone)]
pub struct LocalConfig {
  pub directory: Dir<'static>,
}

/// Configuration for a remote (i.e hosted) website/web app
#[derive(Debug, Clone)]
pub struct RemoteConfig {
  pub url: String,
}

/// This is so the Window::new() can just be provided a LocalConfig or RemoteConfig
/// and it will automatically create the correct ContentConfig
pub trait IntoContentConfig {
  fn into_content_config(self) -> ContentConfig;
}

impl IntoContentConfig for LocalConfig {
  fn into_content_config(self) -> ContentConfig {
    ContentConfig::Local(self)
  }
}

impl IntoContentConfig for RemoteConfig {
  fn into_content_config(self) -> ContentConfig {
    ContentConfig::Remote(self)
  }
}

/// The main Window, representing a browser window
///
/// This struct is used to create and manage a browser window.
/// It contains all configuration, controls, etc. needed to control the window.
///
/// # Example
/// ```rust
/// let mut win = Window::new(RemoteConfig {
///   url: "https://example.com".to_string(),
/// }, None, PathBuf::from("/path/to/your/app/profiles"))?;
///
/// // By default, the window will be created with the "best" browser, but you can also specify a browser, or even construct a custom one!
/// win.set_browser(Browser {
///   name: "my_browser ",
///   kind: BrowserKind::Chromium,
///   win: BrowserWindowsConfig {
///     paths: vec![PathBuf::from("/path/to/my/browser/executable")],
///     registry_keys: vec![],
///   },
///   unix: vec!["my_browser", "my_browser-browser"],
///   mac: vec![PathBuf::from("/Applications/My Browser.app/Contents/MacOS/my_browser")],
/// })?;
///
/// // This will block the thread until the window is closed
/// win.create()?;
/// ```
#[derive(Debug)]
pub struct Window {
  created: bool,

  webserver_port: Option<u16>,

  config: ContentConfig,
  browser: Browser,

  profile_directory: PathBuf,
  process_handle: Option<SharedChild>,

  // Window properties
  width: u32,
  height: u32,

  initialization_script: String,

  disable_hardware_acceleration: bool,

  firefox_config: Option<FirefoxConfig>,
  chromium_config: Option<ChromiumConfig>,

  ipc: Arc<Mutex<Option<ipc::BrowserIpc>>>,
}

impl Window {
  /// Create a new window with the specified browser engine (if any) and profile directory.
  pub fn new(
    config: impl IntoContentConfig,
    engine: Option<BrowserKind>,
    profile_directory: PathBuf,
  ) -> Result<Self, CrowserError> {
    let browser = match browser::get_best_browser(engine) {
      Some(browser) => browser,
      None => {
        return Err(CrowserError::NoBrowser(
          "No compatible browsers on system!".to_string(),
        ))
      }
    };

    Ok(Self {
      profile_directory,

      webserver_port: None,

      process_handle: None,

      created: false,

      config: config.into_content_config(),
      browser,

      width: 800,
      height: 600,

      initialization_script: "".to_string(),

      disable_hardware_acceleration: false,

      firefox_config: None,
      chromium_config: None,

      ipc: Arc::new(Mutex::new(None)),
    })
  }

  /// Set the remote URL for the window, if it is configured to be remote.
  pub fn set_url(&mut self, url: impl AsRef<str>) -> Result<(), CrowserError> {
    match &mut self.config {
      ContentConfig::Remote(remote) => {
        remote.url = url.as_ref().to_string();
      }
      _ => {},
    }

    let ipc = self.get_ipc();
    let mut ipc = ipc.lock().unwrap();
    
    if let Some(ipc) = ipc.as_mut() {
      // TODO this feels wack, there is probably a CDP way to do this
      ipc.eval(format!("window.location.href = '{}'", url.as_ref()))?;
    }

    Ok(())
  }

  /// Manually set the browser to use for the window.
  pub fn set_browser(&mut self, browser: Browser) -> Result<(), CrowserError> {
    if self.created {
      return Err(CrowserError::DoAfterCreate(
        "Cannot set browser after window is created".to_string(),
      ));
    }

    self.browser = browser;

    Ok(())
  }

  /// Set the window size
  pub fn set_size(&mut self, width: u32, height: u32) {
    self.width = width;
    self.height = height;
  }

  /// Set the initialization script for the window.
  /// This script will be run when the window is created or the contents are reloaded.
  pub fn set_initialization_script(&mut self, script: impl AsRef<str>) -> Result<(), CrowserError> {
    if self.created {
      return Err(CrowserError::DoAfterCreate(
        "Initialization script will have no effect if window is already created".to_string(),
      ));
    }

    self.initialization_script = script.as_ref().to_string();

    Ok(())
  }

  /// Get a clone of the IPC handler, so you can use it in threads without bringing along the whole Window struct.
  pub fn get_ipc(&self) -> Arc<Mutex<Option<ipc::BrowserIpc>>> {
    self.ipc.clone()
  }

  /// Disable hardware acceleration in the browser window.
  pub fn disable_hardware_acceleration(&mut self) -> Result<(), CrowserError> {
    if self.created {
      return Err(CrowserError::DoAfterCreate(
        "Changing hardware acceleration will have no effect if window is already created"
          .to_string(),
      ));
    }

    self.disable_hardware_acceleration = true;

    Ok(())
  }

  /// Set Firefox-specific configuration options. This will have no effect if the window is not a Firefox window.
  pub fn set_firefox_config(&mut self, config: FirefoxConfig) -> Result<(), CrowserError> {
    if self.created {
      return Err(CrowserError::DoAfterCreate(
        "Changing Firefox-specific configuration will have no effect if window is already created"
          .to_string(),
      ));
    }

    self.firefox_config = Some(config);

    Ok(())
  }

  /// Set Chromium-specific configuration options. This will have no effect if the window is not a Chromium window.
  pub fn set_chromium_config(&mut self, config: ChromiumConfig) -> Result<(), CrowserError> {
    if self.created {
      return Err(CrowserError::DoAfterCreate(
        "Changing Chromium-specific configuration will have no effect if window is already created"
          .to_string(),
      ));
    }

    self.chromium_config = Some(config);

    Ok(())
  }

  /// Create the window after you have provided all the necessary configuration.
  pub fn create(&mut self) -> Result<(), CrowserError> {
    self.created = true;

    let port = get_available_port(Some(9000));
    self.webserver_port = Some(port);

    let t_config = self.config.clone();
    let (w_tx, w_rx) = std::sync::mpsc::channel::<WebserverMessage>();
    let webserver_thread = std::thread::spawn(move || {
      if let ContentConfig::Local(config) = t_config {
        let webserver = Webserver::new(port, config.directory);

        println!("Webserver started on port {}", port);

        if let Ok(webserver) = webserver {
          loop {
            // Small delay to prevent a tight loop
            std::thread::sleep(std::time::Duration::from_millis(1));

            if let Ok(WebserverMessage::Kill) = w_rx.try_recv() {
              break;
            }

            match webserver.poll_request() {
              Ok(_) => {}
              Err(err) => {
                eprintln!("Webserver error: {}", err);
                break;
              }
            };
          }
        }
      }
    });

    let browser_path = get_browser_path(&self.browser);

    if browser_path.is_none() {
      return Err(CrowserError::NoBrowser(
        "No compatible browsers on system! I don't know how you got this far...".to_string(),
      ));
    }

    let browser_path = browser_path.unwrap();
    let mut cmd: std::process::Command = std::process::Command::new(browser_path);
    let mut args = match self.browser.kind {
      BrowserKind::Chromium => browser::chromium::generate_cli_options(self),
      BrowserKind::Gecko => browser::firefox::generate_cli_options(self),
      _ => {
        vec![]
      }
    };
    let remote_debugging_port = get_available_port(Some(8000));

    args.push("--remote-debugging-port=".to_string() + &remote_debugging_port.to_string());

    cmd.args(args);

    match self.browser.kind {
      BrowserKind::Chromium => browser::chromium::write_extra_profile_files(self)?,
      BrowserKind::Gecko => browser::firefox::write_extra_profile_files(self)?,
      _ => {}
    }

    let process = cmd.spawn()?;
    let terminated = Arc::new(AtomicBool::new(false));

    self.process_handle = Some(SharedChild::new(process)?);

    // Now that the process is running, we can start attempting to connect to it with IPC
    let ipc = ipc::BrowserIpc::new(remote_debugging_port)?;
    self.ipc.lock().unwrap().replace(ipc);

    for signal in &[signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM] {
      let terminated = terminated.clone();
      signal_hook::flag::register(*signal, terminated)?;
    }

    loop {
      std::thread::sleep(std::time::Duration::from_secs(1));

      if terminated.load(std::sync::atomic::Ordering::Relaxed) {
        // Kill the process
        if let Some(child) = self.process_handle.as_ref() {
          child.kill()?;
        }

        match w_tx.send(WebserverMessage::Kill) {
          Ok(_) => {}
          Err(_) => {
            // TODO This likely means the thread is already dead
          }
        }

        webserver_thread.join()?;
        break;
      }

      // if the process is dead, break
      if let Some(child) = self.process_handle.as_ref() {
        if child.try_wait()?.is_some() {
          match w_tx.send(WebserverMessage::Kill) {
            Ok(_) => {}
            Err(_) => {
              // TODO This likely means the thread is already dead
            }
          }

          webserver_thread.join()?;
          break;
        }
      } else {
        break;
      }
    }

    // If we have broken out of the loop, the window is closed
    self.created = false;

    Ok(())
  }

  /// Force kill the window. The death of the window will be detected and kill the webserver, if running a local configuration.
  pub fn kill(&mut self) -> Result<(), CrowserError> {
    if !self.created {
      return Err(CrowserError::DoBeforeCreate(
        "Cannot kill window before it is created".to_string(),
      ));
    }

    if let Some(child) = self.process_handle.as_ref() {
      child.kill()?;
    }

    Ok(())
  }

  /// Wipe the profile directory for the window. This will remove all user data, settings, etc. for the window.
  pub fn clear_profile(&mut self) -> Result<(), CrowserError> {
    if self.created {
      return Err(CrowserError::DoAfterCreate(
        "Cannot reset profile after window is created".to_string(),
      ));
    }

    std::fs::remove_dir_all(&self.profile_directory)?;

    Ok(())
  }
}
