use std::{
  path::PathBuf,
  sync::{atomic::AtomicBool, Arc},
};

use browser::{Browser, BrowserKind};
use error::CrowserError;
use shared_child::SharedChild;

pub mod browser;
mod error;
mod webserver;

#[derive(Debug)]
pub struct FirefoxConfig {
  custom_css: Option<String>,
}

#[derive(Debug)]
pub struct ChromiumConfig {
  // TODO
}

#[derive(Debug, Clone)]
pub enum ContentConfig {
  Local(LocalConfig),
  Remote(RemoteConfig),
}

#[derive(Debug, Clone)]
pub struct LocalConfig {
  pub port_range: (u16, u16),
  pub directory: PathBuf,
  port: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct RemoteConfig {
  pub url: String,
}

// Create a trait so the Window::new() can just be provided a LocalConfig or RemoteConfig
// and it will automatically create the correct ContentConfig
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

#[derive(Debug)]
pub struct Window {
  created: bool,

  config: ContentConfig,
  browser: (Browser, PathBuf),

  profile_directory: PathBuf,
  process_handle: Option<SharedChild>,

  // Window properties
  width: u32,
  height: u32,

  initialization_script: String,

  disable_hardware_acceleration: bool,

  firefox_config: Option<FirefoxConfig>,
  chromium_config: Option<ChromiumConfig>,
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
    })
  }

  pub fn set_url(&mut self, url: impl AsRef<str>) -> Result<(), CrowserError> {
    match &mut self.config {
      ContentConfig::Remote(remote) => {
        remote.url = url.as_ref().to_string();
      }
      _ => {
        return Err(CrowserError::DoAfterCreate(
          "Cannot set URL after window is created".to_string(),
        ))
      }
    }

    // TODO If we are already created, we need to send the signal to the window to change the URL
    Ok(())
  }

  pub fn set_size(&mut self, width: u32, height: u32) {
    self.width = width;
    self.height = height;
  }

  pub fn set_initialization_script(&mut self, script: impl AsRef<str>) -> Result<(), CrowserError> {
    if self.created {
      return Err(CrowserError::DoAfterCreate(
        "Initialization script will have no effect if window is already created".to_string(),
      ));
    }

    self.initialization_script = script.as_ref().to_string();

    Ok(())
  }

  pub fn disable_hardware_acceleration(&mut self) -> Result<(), CrowserError> {
    if self.created {
      return Err(CrowserError::DoAfterCreate(
        "Initialization script will have no effect if window is already created".to_string(),
      ));
    }

    self.disable_hardware_acceleration = true;

    Ok(())
  }

  pub fn set_firefox_config(&mut self, config: FirefoxConfig) -> Result<(), CrowserError> {
    if self.created {
      return Err(CrowserError::DoAfterCreate(
        "Initialization script will have no effect if window is already created".to_string(),
      ));
    }

    self.firefox_config = Some(config);

    Ok(())
  }

  pub fn set_chromium_config(&mut self, config: ChromiumConfig) -> Result<(), CrowserError> {
    if self.created {
      return Err(CrowserError::DoAfterCreate(
        "Initialization script will have no effect if window is already created".to_string(),
      ));
    }

    self.chromium_config = Some(config);

    Ok(())
  }

  pub fn create(&mut self) -> Result<(), CrowserError> {
    self.created = true;

    // TODO this needs to provide CLI options and crap
    let mut cmd = std::process::Command::new(self.browser.1.clone());

    cmd.args(match self.browser.0.kind {
      BrowserKind::Chromium => browser::chromium::generate_cli_options(self),
      BrowserKind::Gecko => browser::firefox::generate_cli_options(self),
      _ => {
        vec![]
      }
    });

    if self.browser.0.kind == BrowserKind::Gecko {
      browser::firefox::write_extra_profile_files(self)?;
    }

    let process = cmd.spawn()?;
    let terminated = Arc::new(AtomicBool::new(false));

    self.process_handle = Some(SharedChild::new(process)?);

    for signal in &[signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM] {
      let terminated = terminated.clone();
      signal_hook::flag::register(*signal, terminated)?;
    }

    // TODO create like an event handler and stuff
    loop {
      std::thread::sleep(std::time::Duration::from_secs(1));

      if terminated.load(std::sync::atomic::Ordering::Relaxed) {
        // Kill the process
        self.process_handle.as_ref().unwrap().kill()?;
        break;
      }

      // if the process is dead, break
      if self.process_handle.as_ref().unwrap().try_wait()?.is_some() {
        break;
      }
    }

    Ok(())
  }

  pub fn kill(&mut self) -> Result<(), CrowserError> {
    if !self.created {
      return Err(CrowserError::DoBeforeCreate(
        "Cannot kill window before it is created".to_string(),
      ));
    }

    self.process_handle.as_ref().unwrap().kill()?;

    Ok(())
  }

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
