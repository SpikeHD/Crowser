#[derive(Debug)]
pub enum CrowserError {
  IoError(std::io::Error),
  NoBrowser(String),
  DoAfterCreate(String),
  DoBeforeCreate(String),
}

impl std::error::Error for CrowserError {}

impl From <std::io::Error> for CrowserError {
  fn from(err: std::io::Error) -> Self {
    CrowserError::IoError(err)
  }
}

impl std::fmt::Display for CrowserError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      CrowserError::IoError(err) => write!(f, "IO Error: {}", err),
      CrowserError::NoBrowser(msg) => write!(f, "No browser found: {}", msg),
      CrowserError::DoAfterCreate(msg) => write!(f, "Do after create error: {}", msg),
      CrowserError::DoBeforeCreate(msg) => write!(f, "Do before create error: {}", msg),
    }
  }
}