use crowser::{self, browser::get_best_browser};

pub fn main() {
  println!("All browsers available on system:");
  for browser in crowser::browser::get_all_existing_browsers() {
    println!("{:?} ({:?})", browser.0.name, browser.0.kind);
  }

  let best = get_best_browser(None).unwrap_or_else(|| {
    println!("No compatible browsers on system!");
    std::process::exit(1)
  });

  println!("Best browser on system:");
  println!(
    "{:?} ({:?}, located at {:?})",
    best.0.name, best.0.kind, best.1
  );
}
