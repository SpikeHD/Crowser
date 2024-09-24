use std::fmt::Debug;

use crate::{
  cdp::{
    self,
    commands::{CDPCommand, RuntimeEvaluate, TargetGetTargets},
    CDP,
  },
  error::CrowserError,
};

type NoArgs = Option<serde_json::Value>;

pub struct BrowserIpc {
  cdp: CDP,
}

// Trait type so Browser can implement debug
impl Debug for BrowserIpc {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "BrowserIpc")
  }
}

impl BrowserIpc {
  pub fn new(port: u16) -> Result<Self, CrowserError> {
    let cdp = cdp::launch(port)?;

    Ok(BrowserIpc { cdp })
  }

  // fn get_tab(&self) -> Result<Arc<Tab>, CrowserError> {
  //   // TODO
  // }

  pub fn eval(&mut self, script: &str) -> Result<(), CrowserError> {
    let params = RuntimeEvaluate {
      expression: script.to_string(),
    };
    let cmd = CDPCommand::new("Runtime.evaluate", params);

    self.cdp.send(cmd)?;

    Ok(())
  }
}
