use std::{fmt::Debug, sync::Arc};

use headless_chrome::{Browser, Tab};

use crate::error::CrowserError;

pub struct BrowserIpc {
  browser: Browser,
}

// Trait type so Browser can implement debug
impl Debug for BrowserIpc {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "BrowserIpc")
  }
}

impl BrowserIpc {
  pub fn new(port: u16) -> Result<Self, CrowserError> {
    let mut ws = String::new();

    // Over the next 5 seconds, attempt to connect to the browser
    for _ in 0..50 {
      // Attempt to get the websocket URL
      match get_ws_url(port) {
        Ok(val) => {
          ws = val;
          break;
        },
        Err(_) => std::thread::sleep(std::time::Duration::from_millis(100)),
      }
    }

    if ws.is_empty() {
      return Err(CrowserError::CDPError("No browser instance to connect to".to_string()));
    }

    match Browser::connect(ws) {
      Ok(browser) => Ok(BrowserIpc {
        browser,
      }),
      Err(err) => return Err(CrowserError::CDPError(err.to_string())),
    }
  }

  fn get_tab(&self) -> Result<Arc<Tab>, CrowserError> {
    // There should only ever be one tab, so just get the first one
    let tabs = self.browser.get_tabs()
      .lock()
      .unwrap();
    let tab = tabs.first();

    if let Some(tab) = tab {
      return Ok(tab.clone());
    }
    
    return Err(CrowserError::NoBrowser("No browser found".to_string()));
  }

  pub fn eval(&self, script: &str) -> Result<(), CrowserError> {
    let tab = self.get_tab()?;
    
    match tab.evaluate(script, true) {
      Ok(_) => Ok(()),
      Err(err) => Err(CrowserError::CDPError(err.to_string())),
    }
  }
}

pub fn get_ws_url(port: u16) -> Result<String, CrowserError> {
  // Make a request to the browsers initial HTTP server to get the websocket URL
  let url = format!("http://127.0.0.1:{}/json/version", port);
  let resp = minreq::get(url).send()?;

  let val = resp.as_str()?;
  let val = val.split("\"webSocketDebuggerUrl\":").collect::<Vec<&str>>()[1];
  let val = val.split("}").collect::<Vec<&str>>()[0];
  let val = val.trim().replace('\"', "");

  Ok(val.to_string())
}