use std::fmt::Display;

use serde::{Deserialize, Serialize};

// "Master" struct that handles the structure of all commands
#[derive(Serialize, Deserialize)]
pub struct CDPCommand {
  pub method: String,
  pub params: serde_json::Value,
}

impl CDPCommand {
  pub fn new(method: impl AsRef<str>, params: impl Serialize) -> Self {
    CDPCommand {
      method: method.as_ref().to_string(),
      params: serde_json::to_value(params).unwrap(),
    }
  }
}

impl From<String> for CDPCommand {
  fn from(val: String) -> Self {
    serde_json::from_str(&val).unwrap()
  }
}

impl Into<String> for CDPCommand {
  fn into(self) -> String {
    serde_json::to_string(&self).unwrap()
  }
}

impl Display for CDPCommand {
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
