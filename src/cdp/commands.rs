/**
 * You may be wondering: why are you manually creating these commands? Isn't there a library for this?
 * 
 * There sure is, several even! However, to keep this project as light as possible, I decided to only implement the commands I needed.
 * I highly doubt I will need like, 90% of the available commands anyways, so this is almost definitely the best way to go. I initially
 * tried this with a CDP crate and it bloated the binary up by several megabytes! No thanks!
 */
use std::fmt::Display;

use serde::{Deserialize, Serialize};

// "Master" struct that handles the structure of all commands
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CDPCommand {
  pub method: String,
  pub params: serde_json::Value,
  pub session_id: Option<String>,
}

impl CDPCommand {
  pub fn new(method: impl AsRef<str>, params: impl Serialize, session_id: Option<String>) -> Self {
    CDPCommand {
      method: method.as_ref().to_string(),
      params: serde_json::to_value(params).unwrap(),
      session_id,
    }
  }
}

impl From<String> for CDPCommand {
  fn from(val: String) -> Self {
    serde_json::from_str(&val).unwrap()
  }
}

impl From<CDPCommand> for String {
  fn from(val: CDPCommand) -> Self {
    serde_json::to_string(&val).unwrap()
  }
}

impl Display for CDPCommand {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", serde_json::to_string(self).unwrap())
  }
}

#[derive(Serialize, Deserialize)]
pub struct CDPResponse {
  pub id: Option<u64>,
  pub result: Option<serde_json::Value>,
}

impl From<String> for CDPResponse {
  fn from(val: String) -> Self {
    serde_json::from_str(&val).unwrap()
  }
}

impl From<CDPResponse> for String {
  fn from(val: CDPResponse) -> Self {
    serde_json::to_string(&val).unwrap()
  }
}

impl Display for CDPResponse {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", serde_json::to_string(self).unwrap())
  }
}

#[derive(Serialize, Deserialize)]
pub struct PageEnable {}

#[derive(Serialize, Deserialize)]
pub struct PageDisable {}

#[derive(Serialize, Deserialize, Debug)]
pub struct RuntimeEvaluate {
  pub expression: String,
}

#[derive(Serialize, Deserialize)]
pub struct TargetSetDiscoverTargets {
  pub discover: bool,
}

#[derive(Serialize, Deserialize)]
pub struct TargetGetTargets {}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetAttachToTarget {
  pub target_id: String,
  pub flatten: bool,
}
