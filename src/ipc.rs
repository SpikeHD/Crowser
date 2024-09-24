use std::fmt::Debug;

use crate::{
  cdp::{
    self,
    commands::{CDPCommand, RuntimeEvaluate, TargetAttachToTarget, TargetGetTargets},
    Cdp,
  },
  error::CrowserError,
};

pub struct BrowserIpc {
  cdp: Cdp,
  session_id: String,
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
    let mut ipc = BrowserIpc {
      cdp,
      session_id: String::new(),
    };

    ipc.attach()?;

    Ok(ipc)
  }

  // fn get_tab(&self) -> Result<Arc<Tab>, CrowserError> {
  //   // TODO
  // }

  pub fn attach(&mut self) -> Result<(), CrowserError> {
    // Get targets
    let t_params = TargetGetTargets {};
    let t_cmd = CDPCommand::new("Target.getTargets", t_params, None);
    let result = self.cdp.send(t_cmd, None)?;
    let result = result.get("result");

    let targets = match result {
      Some(val) => val,
      None => return Err(CrowserError::CDPError("No result found".to_string())),
    };

    let targets = match targets.get("targetInfos") {
      Some(val) => val,
      None => return Err(CrowserError::CDPError("No targets found".to_string())),
    }
    .as_array();

    if let Some(targets) = targets {
      for target in targets {
        let t = target["type"].as_str().unwrap_or_default();

        if t != "page" {
          continue;
        }

        let t_params = TargetAttachToTarget {
          target_id: target["targetId"].as_str().unwrap_or_default().to_string(),
          flatten: true,
        };
        let t_cmd = CDPCommand::new("Target.attachToTarget", t_params, None);
        let result = self.cdp.send(t_cmd, None)?;
        // This returns as CDPCommand
        let result = CDPCommand::from(result.to_string());
        let session_id = result.params.get("sessionId");

        if let Some(session_id) = session_id {
          self.session_id = session_id.as_str().unwrap_or_default().to_string();
          break;
        }
      }
    }

    // Runtime.enable
    // This is a fix for Firefox
    // lol: https://bugzilla.mozilla.org/show_bug.cgi?id=1623482#c12
    let cmd = CDPCommand::new(
      "Runtime.enable",
      serde_json::Value::Null,
      Some(self.session_id.clone()),
    );
    self.cdp.send(cmd, None)?;

    Ok(())
  }

  pub fn eval(&mut self, script: &str) -> Result<String, CrowserError> {
    let params = RuntimeEvaluate {
      expression: script.to_string(),
    };
    let cmd = CDPCommand::new("Runtime.evaluate", params, Some(self.session_id.clone()));
    let result = self.cdp.send(cmd, None)?;
    let result = match result.get("result") {
      Some(val) => val,
      None => return Err(CrowserError::CDPError("No result found".to_string())),
    };

    let value = result.get("value").unwrap_or(&serde_json::Value::Null);
    let value = value.as_str().unwrap_or_default();
    Ok(value.to_string())
  }
}
