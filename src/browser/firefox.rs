use std::{fs, path::PathBuf};

pub fn write_profile(app_name: String, profile_directory: PathBuf, initialization_script: String) -> Result<(), std::io::Error> {
  if !profile_directory.exists() {
    fs::create_dir_all(&profile_directory)?;
  }

  Ok(())
}