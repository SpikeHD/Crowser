use std::{collections::HashMap, fmt::Debug};

use serde_json::Value;

use crate::{
  cdp::{
    self,
    commands::{CDPCommand, RuntimeEvaluate, TargetAttachToTarget, TargetGetTargets},
    Cdp,
  },
  error::CrowserError,
  util::javascript::IPC_JS,
};

pub struct BrowserIpc {
  cdp: Cdp,
  session_id: String,
  attached: bool,

  commands: HashMap<String, Box<dyn Fn(Value) -> Result<Value, CrowserError> + Send + Sync>>,
}

impl Debug for BrowserIpc {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    // We can't properly debug Box<dyn FN> so we just show the keys, which is certainly better than nothing
    let keys = self.commands.keys().collect::<Vec<&String>>();

    f.debug_struct("BrowserIpc")
      .field("cdp", &self.cdp)
      .field("session_id", &self.session_id)
      .field("attached", &self.attached)
      .field("commands", &keys)
      .finish()
  }
}

impl BrowserIpc {
  pub fn new(port: u16) -> Result<Self, CrowserError> {
    let cdp = cdp::launch(port)?;
    let mut ipc = BrowserIpc {
      cdp,
      session_id: String::new(),
      attached: false,

      commands: HashMap::new(),
    };

    ipc.attach()?;

    // Once attached we need to inject the IPC script
    ipc.eval(IPC_JS)?;

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
      None => {
        return Err(CrowserError::CDPError(
          "Attach: No result found".to_string(),
        ))
      }
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
        self.cdp.send(t_cmd, None)?;

        // This triggers the Target.attachedToTarget event
        let evt_result = self.cdp.wait_for_event("Target.attachedToTarget", None)?;
        let evt_result = evt_result.params.get("sessionId");

        if let Some(session_id) = evt_result {
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

    self.attached = true;

    Ok(())
  }

  pub fn wait_until_attached(&mut self) -> Result<(), CrowserError> {
    #[allow(clippy::while_immutable_condition)]
    while !self.attached {
      std::thread::sleep(std::time::Duration::from_millis(100));
    }

    Ok(())
  }

  pub fn eval(&mut self, script: impl AsRef<str>) -> Result<Value, CrowserError> {
    self.wait_until_attached()?;

    let params = RuntimeEvaluate {
      expression: script.as_ref().to_string(),
    };
    let cmd = CDPCommand::new("Runtime.evaluate", params, Some(self.session_id.clone()));
    let result = self.cdp.send(cmd, None)?;
    let res_type = result["result"]["type"].as_str().unwrap_or_default();

    if ["string", "number", "boolean", "bigint", "symbol"].contains(&res_type) {
      match result["result"]["result"].get("value") {
        Some(val) => val,
        None => {
          return Err(CrowserError::CDPError(format!(
            "Eval: No result found in object: {:?}",
            result
          )))
        }
      };
    }

    Ok(Value::Null)
  }

  pub fn register_command(&mut self, name: impl AsRef<str>, callback: Box<dyn Fn(Value) -> Result<Value, CrowserError> + Send + Sync>) -> Result<(), CrowserError> {
    // Check if command already exists
    if self.commands.contains_key(name.as_ref()) {
      // Throw error
      return Err(CrowserError::IpcError("Command already exists".to_string()));
    }

    self.commands.insert(name.as_ref().to_string(), callback);

    Ok(())
  }
}
