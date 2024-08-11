use std::any::Any;

use crate::webserver::WebserverMessage;

#[derive(Debug)]
pub enum CrowserError {
  Generic(Box<dyn Any + Send>),
  IoError(std::io::Error),
  WebserverSendError(std::sync::mpsc::SendError<WebserverMessage>),
  WebserverRecvError(std::sync::mpsc::RecvError),
  NoBrowser(String),
  NoTab(String),
  DoAfterCreate(String),
  DoBeforeCreate(String),
  WebserverError(String),
  CDPError(String),
  WebRequestError(minreq::Error),
  Unknown(()),
}

impl std::error::Error for CrowserError {}

impl From<Box<dyn Any + Send>> for CrowserError {
  fn from(err: Box<dyn Any + Send>) -> Self {
    CrowserError::Generic(err)
  }
}

impl From<std::io::Error> for CrowserError {
  fn from(err: std::io::Error) -> Self {
    CrowserError::IoError(err)
  }
}

impl From<std::sync::mpsc::SendError<WebserverMessage>> for CrowserError {
  fn from(err: std::sync::mpsc::SendError<WebserverMessage>) -> Self {
    CrowserError::WebserverSendError(err)
  }
}

impl From<std::sync::mpsc::RecvError> for CrowserError {
  fn from(err: std::sync::mpsc::RecvError) -> Self {
    CrowserError::WebserverRecvError(err)
  }
}

impl From<minreq::Error> for CrowserError {
  fn from(err: minreq::Error) -> Self {
    CrowserError::WebRequestError(err)
  }
}

impl From<()> for CrowserError {
  fn from(_: ()) -> Self {
    CrowserError::Unknown(())
  }
}

impl std::fmt::Display for CrowserError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      CrowserError::Generic(err) => write!(f, "Generic error: {:?}", err),
      CrowserError::IoError(err) => write!(f, "IO Error: {}", err),
      CrowserError::WebserverSendError(err) => write!(f, "Webserver send error: {}", err),
      CrowserError::WebserverRecvError(err) => write!(f, "Webserver receive error: {}", err),
      CrowserError::NoBrowser(msg) => write!(f, "No browser found: {}", msg),
      CrowserError::NoTab(msg) => write!(f, "No tabs found: {}", msg),
      CrowserError::DoAfterCreate(msg) => write!(f, "Do after create error: {}", msg),
      CrowserError::DoBeforeCreate(msg) => write!(f, "Do before create error: {}", msg),
      CrowserError::WebserverError(msg) => write!(f, "Webserver error: {}", msg),
      CrowserError::CDPError(msg) => write!(f, "CDP error: {}", msg),
      CrowserError::WebRequestError(err) => write!(f, "Web request error: {}", err),
      CrowserError::Unknown(_) => write!(f, "Unknown error"),
    }
  }
}
