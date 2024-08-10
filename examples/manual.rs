use crowser::{browser::get_all_existing_browsers, error::CrowserError, RemoteConfig, Window};

fn main() -> Result<(), CrowserError> {
  let mut profile_dir = std::env::current_dir()?;
  profile_dir.push("example_profiles");

  let config = RemoteConfig {
    url: "https://example.com".to_string(),
  };

  // Engine provided doesn't matter, we provide one manually in a moment
  let mut win = Window::new(config, None, profile_dir)?;
  let the_best_browser_ever = get_all_existing_browsers().into_iter().find(|b| b.name == "edge");

  // If you want the browser's path, you can call:
  // let path = get_browser_path(&the_best_browser_ever.unwrap());

  if let Some(browser) = the_best_browser_ever {
    win.set_browser(browser)?;
  } else {
    println!("This system only has lame browsers!");
    std::process::exit(1);
  }

  win.clear_profile()?;

  win.create()?;

  Ok(())
}
