use commands::CDPCommand;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tungstenite::Message;

use crate::error::CrowserError;

pub mod commands;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CDPMessageInternal {
  id: u64,
  method: String,
  params: serde_json::Value,
  #[serde(skip_serializing_if = "Option::is_none")]
  session_id: Option<String>,
}

impl CDPMessageInternal {
  fn new(id: u64, cmd: CDPCommand) -> Self {
    CDPMessageInternal {
      id,
      method: cmd.method,
      params: cmd.params,
      session_id: cmd.session_id,
    }
  }
}

struct CDPMessenger {
  tx: flume::Sender<String>,
  rx: flume::Receiver<String>,
}

pub struct Cdp {
  cmd_id: u64,

  cmd: CDPMessenger,
  ws: CDPMessenger,
}

/// TODO: Lots needs to be done here:
///
/// - Create two message queues, one for command results and one for events
/// - Enforce syncronization of the command results (ie if two are run in succession, they may desync)
///   - This will probs be done by using a hashmap to store command IDs and results and such. Thank you CDP for providing those IDs lol
impl Cdp {
  pub fn new() -> Self {
    let (cmd_tx, cmd_rx) = flume::unbounded();
    let (ws_tx, ws_rx) = flume::unbounded();

    Cdp {
      cmd_id: 0,
      cmd: CDPMessenger {
        tx: cmd_tx,
        rx: cmd_rx,
      },
      ws: CDPMessenger {
        tx: ws_tx,
        rx: ws_rx,
      },
    }
  }

  pub fn connect(&mut self, port: u16) -> Result<(), CrowserError> {
    // Spend a few second trying to get the WS URL
    let mut ws_url = String::new();

    for _ in 0..50 {
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
    let tx = self.ws.tx.clone();

    std::thread::spawn(move || ws_executor(ws_url, tx, rx));

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
    let msg = serde_json::to_string(&CDPMessageInternal::new(self.cmd_id + 1, cmd));
    let msg = msg.map_err(|e| {
      CrowserError::CDPError("Could not serialize message: ".to_string() + &e.to_string())
    })?;

    self.cmd_id += 1;

    self.cmd.tx.send(msg).map_err(|e| {
      CrowserError::CDPError("Could not send message: ".to_string() + &e.to_string())
    })?;

    let rx = self.ws.rx.clone();

    // Wait for response to be recieved
    let wait_thread = std::thread::spawn(move || {
      let timeout = timeout.unwrap_or(std::time::Duration::from_secs(1));

      if let Ok(val) = rx.recv_timeout(timeout) {
        return Ok(serde_json::from_str(&val).unwrap_or_default());
      }

      Err(CrowserError::CDPError(
        "Timeout waiting for response".to_string(),
      ))
    });

    match wait_thread.join() {
      Ok(val) => val,
      Err(err) => Err(err.into()),
    }
  }
}

pub fn ws_executor(
  url: impl AsRef<str>,
  tx: flume::Sender<String>,
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
    // This is non-blocking so it should be fine
    let msg = match ws.read() {
      Ok(val) => val,
      Err(_) => Message::Binary(vec![]),
    };
    let cmd = match rx.try_recv() {
      Ok(val) => val,
      Err(_) => String::new(),
    };

    if !cmd.is_empty() {
      ws.send(cmd.into()).map_err(|e| {
        CrowserError::CDPError("Could not send command: ".to_string() + &e.to_string())
      })?;
    }

    if !msg.is_empty() {
      tx.send(msg.to_string())?;
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

  let val = resp.as_str()?;
  let val = val
    .split("\"webSocketDebuggerUrl\":")
    .collect::<Vec<&str>>()[1];
  let val = val.split("}").collect::<Vec<&str>>()[0];
  let val = val.trim().replace('\"', "");

  Ok(val.to_string())
}
