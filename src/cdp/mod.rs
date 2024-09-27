use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
  vec,
};

use commands::CDPCommand;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tungstenite::Message;

use crate::error::CrowserError;

pub mod commands;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CDPMessageInternal {
  id: usize,
  method: String,
  #[serde(skip_serializing_if = "Value::is_null")]
  params: serde_json::Value,
  #[serde(skip_serializing_if = "Option::is_none")]
  session_id: Option<String>,
}

impl CDPMessageInternal {
  fn new(id: usize, cmd: CDPCommand) -> Self {
    CDPMessageInternal {
      id,
      method: cmd.method,
      params: cmd.params,
      session_id: cmd.session_id,
    }
  }
}

#[derive(Debug, Clone)]
struct CDPMessenger {
  tx: flume::Sender<String>,
  rx: flume::Receiver<String>,
}

#[derive(Debug, Clone)]
struct CDPMessage {
  result: Value,
}

#[derive(Debug, Clone)]
struct CDPIpcManager {
  session_messages: HashMap<String, CDPMessage>,
  events: Vec<CDPCommand>,
}

#[derive(Debug, Clone)]
pub struct Cdp {
  cmd_id: usize,
  cmd: CDPMessenger,
  manager: Arc<Mutex<CDPIpcManager>>,
}

impl Cdp {
  pub fn new() -> Self {
    let (cmd_tx, cmd_rx) = flume::unbounded();

    Cdp {
      cmd_id: 0,
      cmd: CDPMessenger {
        tx: cmd_tx,
        rx: cmd_rx,
      },
      manager: Arc::new(Mutex::new(CDPIpcManager {
        session_messages: HashMap::new(),
        events: Vec::new(),
      })),
    }
  }

  pub fn connect(&mut self, port: u16) -> Result<(), CrowserError> {
    // Spend a few second trying to get the WS URL
    let mut ws_url = String::new();

    // 1 minute wait
    for _ in 0..600 {
      match get_ws_url(port) {
        Ok(val) => {
          ws_url = val.trim().to_string();
          break;
        }
        Err(_) => std::thread::sleep(std::time::Duration::from_millis(100)),
      }
    }

    if ws_url.is_empty() {
      return Err(CrowserError::CDPError(
        "No browser instance to connect to".to_string(),
      ));
    }

    let rx = self.cmd.rx.clone();
    let manager = self.manager.clone();

    std::thread::spawn(move || ws_executor(manager, ws_url, rx));

    Ok(())
  }

  // pub fn poll(&mut self) -> Result<String, CrowserError> {
  //   self
  //     .ws
  //     .rx
  //     .recv()
  //     .map_err(|_| CrowserError::CDPError("Could not receive message".to_string()))
  // }

  pub fn send(
    &mut self,
    cmd: CDPCommand,
    timeout: Option<std::time::Duration>,
  ) -> Result<Value, CrowserError> {
    let manager = self.manager.clone();
    let msg = serde_json::to_string(&CDPMessageInternal::new(self.cmd_id + 1, cmd));
    let msg = msg.map_err(|e| {
      CrowserError::CDPError("Could not serialize message: ".to_string() + &e.to_string())
    })?;

    self.cmd_id += 1;

    self.cmd.tx.send(msg).map_err(|e| {
      CrowserError::CDPError("Could not send message: ".to_string() + &e.to_string())
    })?;

    let id = self.cmd_id;
    // Wait for response to be recieved
    let wait_thread = std::thread::spawn(move || {
      let timeout = timeout.unwrap_or(std::time::Duration::from_secs(1));
      let now = std::time::Instant::now();

      // Wait for a response in the manager using try_lock and the timeout
      loop {
        match manager.try_lock() {
          Ok(manager) => {
            if let Some(msg) = manager.session_messages.get(&id.to_string()) {
              return Ok(msg.result.clone());
            }
          }
          Err(_) => std::thread::sleep(std::time::Duration::from_millis(100)),
        }

        if now.elapsed().as_millis() > timeout.as_millis() {
          return Err(CrowserError::CDPError(
            "Timeout waiting for response".to_string(),
          ));
        }
      }
    });

    match wait_thread.join() {
      Ok(val) => val,
      Err(err) => Err(err.into()),
    }
  }

  pub fn events(&mut self) -> Result<Vec<CDPCommand>, CrowserError> {
    let manager = self.manager.clone();
    let manager = manager.lock().unwrap();

    Ok(manager.events.clone())
  }

  pub fn last_event_by_name(&mut self, name: &str) -> Result<Option<CDPCommand>, CrowserError> {
    let events = self.events()?;

    for event in events.iter().rev() {
      if event.method == name {
        return Ok(Some(event.clone()));
      }
    }
    Ok(None)
  }

  // pub fn all_events_by_name(&mut self, name: &str) -> Result<Vec<CDPCommand>, CrowserError> {
  //   let events = self.events();
  //   let mut new_events = vec![];

  //   for event in events? {
  //     if event.method == name {
  //       new_events.push(event);
  //     }
  //   }

  //   Ok(new_events)
  // }

  pub fn wait_for_event(
    &mut self,
    name: &str,
    timeout: Option<std::time::Duration>,
  ) -> Result<CDPCommand, CrowserError> {
    let timeout = timeout.unwrap_or(std::time::Duration::from_secs(1));
    let now = std::time::Instant::now();

    loop {
      std::thread::sleep(std::time::Duration::from_millis(100));

      let mut events = self.events()?;

      for event in events.iter_mut() {
        if event.method == name && !event.seen.unwrap_or(false) {
          event.seen = Some(true);
          return Ok(event.clone());
        }
      }

      if timeout.as_millis() > 0 && now.elapsed().as_millis() > timeout.as_millis() {
        return Err(CrowserError::CDPError(
          "Timeout waiting for event".to_string(),
        ));
      }
    }
  }
}

fn ws_executor(
  manager: Arc<Mutex<CDPIpcManager>>,
  url: impl AsRef<str>,
  rx: flume::Receiver<String>,
) -> Result<(), CrowserError> {
  let (mut ws, _) = match tungstenite::connect(url.as_ref()) {
    Ok(val) => val,
    Err(err) => {
      return Err(CrowserError::CDPError(
        "Could not connect to browser: ".to_string() + &err.to_string(),
      ))
    }
  };

  // Make the socket non-blocking so we can recieve and send in the same loop
  match ws.get_mut() {
    tungstenite::stream::MaybeTlsStream::Plain(val) => val.set_nonblocking(true),
    _ => unimplemented!(),
  }?;

  loop {
    // Small delay to prevent a tight loop
    std::thread::sleep(std::time::Duration::from_millis(1));

    // This is non-blocking so it should be fine
    let msg = match ws.read() {
      Ok(val) => val,
      Err(_) => Message::Binary(vec![]),
    };
    let cmd = rx.try_recv().unwrap_or_default();

    if !cmd.is_empty() {
      ws.send(cmd.into()).map_err(|e| {
        CrowserError::CDPError("Could not send command: ".to_string() + &e.to_string())
      })?;
    }

    if !msg.is_empty() {
      let mut messages = manager.lock().unwrap();
      let msg: Value = serde_json::from_str(&msg.to_string()).unwrap();

      // If it doesn't have an ID, it's an event, otherwise it's a response
      if msg["id"].is_null() {
        messages.events.push(CDPCommand::from(msg.to_string()));
      } else {
        messages
          .session_messages
          .insert(msg["id"].to_string(), CDPMessage { result: msg });
      }
    }
  }
}

pub fn launch(port: u16) -> Result<Cdp, CrowserError> {
  let mut cdp = Cdp::new();
  cdp.connect(port)?;
  Ok(cdp)
}

pub fn get_ws_url(port: u16) -> Result<String, CrowserError> {
  // Make a request to the browsers initial HTTP server to get the websocket URL
  let url = format!("http://127.0.0.1:{}/json/version", port);
  let resp = minreq::get(url).send()?;

  for _ in 0..50 {
    match attempt_get_ws_url(resp.as_str()?) {
      Ok(val) => return Ok(val),
      Err(_) => std::thread::sleep(std::time::Duration::from_millis(100)),
    }
  }

  Err(CrowserError::CDPError(
    "Could not get websocket URL".to_string(),
  ))
}

fn attempt_get_ws_url(contents: impl AsRef<str>) -> Result<String, CrowserError> {
  let val = contents
    .as_ref()
    .split("\"webSocketDebuggerUrl\":")
    .collect::<Vec<&str>>();
  let val = val.get(1);

  if val.is_none() {
    return Err(CrowserError::CDPError(
      "Could not get websocket URL".to_string(),
    ));
  }

  let val = val.unwrap_or(&"").split("}").collect::<Vec<&str>>()[0];
  let val = val.trim().replace('\"', "");

  Ok(val)
}
