use std::{
  collections::HashMap,
  fmt::Debug,
  sync::{Arc, Mutex},
};

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

type IpcRegistrationMap =
  Arc<Mutex<HashMap<String, Vec<Box<dyn Fn(Value) -> Result<Value, CrowserError> + Send + Sync>>>>>;

#[derive(Clone)]
pub struct BrowserIpc {
  cdp: Arc<Mutex<Cdp>>,
  session_id: String,
  attached: bool,

  commands: IpcRegistrationMap,
  listeners: IpcRegistrationMap,
}

impl Debug for BrowserIpc {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    // We can't properly debug Box<dyn FN> so we just show the keys, which is certainly better than nothing
    let c_map = self.commands.lock().unwrap();
    let l_map = self.listeners.lock().unwrap();
    let c_keys = c_map.keys().collect::<Vec<&String>>();
    let l_keys = l_map.keys().collect::<Vec<&String>>();

    f.debug_struct("BrowserIpc")
      .field("cdp", &self.cdp)
      .field("session_id", &self.session_id)
      .field("attached", &self.attached)
      .field("commands", &c_keys)
      .field("listeners", &l_keys)
      .finish()
  }
}

impl BrowserIpc {
  pub fn new(port: u16) -> Result<Self, CrowserError> {
    let cdp = cdp::launch(port)?;
    let mut ipc = BrowserIpc {
      cdp: Arc::new(Mutex::new(cdp)),
      session_id: String::new(),
      attached: false,

      commands: Arc::new(Mutex::new(HashMap::new())),
      listeners: Arc::new(Mutex::new(HashMap::new())),
    };

    ipc.attach()?;
    ipc.event_loop()?;

    ipc.inject();

    Ok(ipc)
  }

  // fn get_tab(&self) -> Result<Arc<Tab>, CrowserError> {
  //   // TODO
  // }

  pub fn attach(&mut self) -> Result<(), CrowserError> {
    let mut cdp = self.cdp.lock().unwrap();
    // Get targets
    let t_params = TargetGetTargets {};
    let t_cmd = CDPCommand::new("Target.getTargets", t_params, None);
    let result = cdp.send(t_cmd, None)?;
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
        cdp.send(t_cmd, None)?;

        // This triggers the Target.attachedToTarget event
        let evt_result = cdp.wait_for_event("Target.attachedToTarget", None)?;
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
    cdp.send(cmd, None)?;

    self.attached = true;

    Ok(())
  }

  fn inject(&mut self) {
    self.eval(IPC_JS).unwrap_or_default();
  }

  /// Non-blocking event loop for tracking CDP events
  fn event_loop(&mut self) -> Result<(), CrowserError> {
    let root_ipc = Arc::new(Mutex::new(self.clone()));

    std::thread::spawn(move || {
      let mut last_context_create_uid = String::new();

      loop {
        std::thread::sleep(std::time::Duration::from_millis(10));

        let ipc = match root_ipc.try_lock() {
          Ok(val) => val,
          Err(_) => continue,
        };
        let mut cdp = match ipc.cdp.try_lock() {
          Ok(val) => val,
          Err(_) => continue,
        };
        // let listeners = match ipc.listeners.try_lock() {
        //   Ok(val) => val,
        //   Err(_) => continue,
        // };

        let refresh = cdp.last_event_by_name("Runtime.executionContextCreated");

        // Drop these explicitly now to ensure the lock is available for the next event
        drop(cdp);
        drop(ipc);

        if let Ok(Some(evt)) = refresh {
          let uid = evt.params["context"]["uniqueId"]
            .as_str()
            .unwrap_or_default();

          if uid != last_context_create_uid {
            last_context_create_uid = uid.to_string();

            // We don't use try_lock here because we NEED to ensure IPC is reinjected right now
            let mut ipc = root_ipc.lock().unwrap();

            // We need to reinject because a new context has been created
            ipc.inject();
          }
        }
      }
    });

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

    let mut cdp = self.cdp.lock().unwrap();
    let params = RuntimeEvaluate {
      expression: script.as_ref().to_string(),
      await_promise: Some(true),
    };
    let cmd = CDPCommand::new("Runtime.evaluate", params, Some(self.session_id.clone()));
    let result = cdp.send(cmd, None)?;
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

  pub fn register_command(
    &mut self,
    name: impl AsRef<str>,
    callback: Box<dyn Fn(Value) -> Result<Value, CrowserError> + Send + Sync>,
  ) -> Result<(), CrowserError> {
    let mut commands = self.commands.lock().unwrap();

    // Check if command already exists
    if commands.contains_key(name.as_ref()) {
      // Throw error
      return Err(CrowserError::IpcError("Command already exists".to_string()));
    }

    commands.insert(name.as_ref().to_string(), vec![callback]);

    Ok(())
  }
}
