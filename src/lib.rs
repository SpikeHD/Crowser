use std::path::PathBuf;

use browser::{Browser, BrowserKind};
use error::CrowserError;

pub mod browser;
mod error;
mod vectrait;

// Browser-specific modules

#[derive(Debug)]
pub struct Window {
  created: bool,

  app_name: String,
  title: String,
  url: String,
  browser: (Browser, PathBuf),

  profile_directory: PathBuf,

  // Window properties
  width: u32,
  height: u32,

  initialization_script: String,
}

impl Window {
  pub fn new(app_name: String, engine: Option<BrowserKind>, profile_directory: PathBuf) -> Result<Self, CrowserError> {
    let browser = match browser::get_best_browser(engine) {
      Some(browser) => browser,
      None => return Err(CrowserError::NoBrowser("No compatible browsers on system!".to_string())),
    };

    Ok(Self {
      app_name,
      profile_directory,

      created: false,

      title: String::from(""),
      url: String::from(""),
      browser,

      width: 800,
      height: 600,

      initialization_script: "".to_string(),
    })
  }

  pub fn set_title(&mut self, title: &str) {
    self.title = title.to_string();
  }

  pub fn set_url(&mut self, url: &str) {
    self.url = url.to_string();

    // TODO If we are already created, we need to send the signal to the window to change the URL
  }

  pub fn set_size(&mut self, width: u32, height: u32) {
    self.width = width;
    self.height = height;
  }

  pub fn set_initialization_script(&mut self, script: &str) -> Result<(), CrowserError> {
    if self.created {
      return Err(CrowserError::DoAfterCreate("Initialization script will have no effect if window is already created".to_string()));
    }

    self.initialization_script = script.to_string();

    Ok(())
  }

  pub fn create(&mut self) -> Result<(), CrowserError> {
    match self.browser.0.kind {
      BrowserKind::Chromium => browser::chromium::write_profile(self.app_name.clone(), self.profile_directory.clone(), self.initialization_script.clone()),
      BrowserKind::Gecko => browser::firefox::write_profile(self.app_name.clone(), self.profile_directory.clone(), self.initialization_script.clone()),
      _ => {
        return Err(CrowserError::NoBrowser(format!("Browser engine {:?} not supported", self.browser)));
      },
    }?;

    self.created = true;

    Ok(())
  }
}

