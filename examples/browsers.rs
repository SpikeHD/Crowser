use crowser::{self, browser::{get_best_browser, get_browser_path}};

pub fn main() {
  println!("All browsers available on system:");
  for browser in crowser::browser::get_all_existing_browsers() {
    println!("{:?} ({:?})", browser.name, browser.kind);
  }

  let best = get_best_browser(None).unwrap_or_else(|| {
    println!("No compatible browsers on system!");
    std::process::exit(1)
  });

  println!("Best browser on system:");
  println!(
    "{:?} ({:?}, located at {:?})",
    best.name, best.kind, get_browser_path(&best)
  );
}
