use std::{path::PathBuf, sync::{atomic::AtomicBool, Arc}};

use browser::{Browser, BrowserKind};
use error::CrowserError;
use shared_child::SharedChild;

pub mod browser;
mod error;

// Browser-specific modules

#[derive(Debug)]
pub struct Window {
  created: bool,

  url: String,
  browser: (Browser, PathBuf),

  profile_directory: PathBuf,
  process_handle: Option<SharedChild>,

  // Window properties
  width: u32,
  height: u32,

  initialization_script: String,

  disable_hardware_acceleration: bool,
}

impl Window {
  pub fn new(engine: Option<BrowserKind>, profile_directory: PathBuf) -> Result<Self, CrowserError> {
    let browser = match browser::get_best_browser(engine) {
      Some(browser) => browser,
      None => return Err(CrowserError::NoBrowser("No compatible browsers on system!".to_string())),
    };

    Ok(Self {
      profile_directory,

      process_handle: None,

      created: false,

      url: String::from(""),
      browser,

      width: 800,
      height: 600,

      initialization_script: "".to_string(),

      disable_hardware_acceleration: false,
    })
  }

  pub fn set_url(&mut self, url: impl AsRef<str>) {
    self.url = url.as_ref().to_string();

    // TODO If we are already created, we need to send the signal to the window to change the URL
  }

  pub fn set_size(&mut self, width: u32, height: u32) {
    self.width = width;
    self.height = height;
  }

  pub fn set_initialization_script(&mut self, script: impl AsRef<str>) -> Result<(), CrowserError> {
    if self.created {
      return Err(CrowserError::DoAfterCreate("Initialization script will have no effect if window is already created".to_string()));
    }

    self.initialization_script = script.as_ref().to_string();

    Ok(())
  }

  pub fn disable_hardware_acceleration(&mut self) -> Result<(), CrowserError> {
    if self.created {
      return Err(CrowserError::DoAfterCreate("Initialization script will have no effect if window is already created".to_string()));
    }

    self.disable_hardware_acceleration = true;

    Ok(())
  }

  pub fn create(&mut self) -> Result<(), CrowserError> {
    self.created = true;

    // TODO this needs to provide CLI options and crap
    let mut cmd = std::process::Command::new(self.browser.1.clone());

    cmd.args(
      match self.browser.0.kind {
        BrowserKind::Chromium => browser::chromium::generate_cli_options(self),
        BrowserKind::Gecko => browser::firefox::generate_cli_options(self),
        _ => {
          vec![]
        },
      }
    );

    let process = cmd.spawn()?;

    self.process_handle = Some(SharedChild::new(process)?);

    let terminated = Arc::new(AtomicBool::new(false));

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
}

