<div align="center">
  <h1>Crowser</h1>
  <p>Create desktop apps using user-supplied browsers</p>
</div>

<div align="center">
  <img src="https://img.shields.io/github/actions/workflow/status/SpikeHD/crowser/check.yml" />
  <img src="https://img.shields.io/github/repo-size/SpikeHD/crowser" />
  <img src="https://img.shields.io/github/commit-activity/m/SpikeHD/crowser" />
</div>

# Features and Support

## Main features

* \>1MB binary size with no external dependencies (except for the browser of course!)
* Multi-platform. Supports Windows, macOS, and Linux.
* Support for both local and remote websites.
* Maximizes performance of whatever browser is chosen, and uses an entirely separate browser profile.

## Browser support

All browser support comes with a few small caveats. You may notice small inconsistencies between, say, your app running in Firefox and in Chrome.

* Chrome/Chromium (stable, beta, dev, canary)
* Edge (stable, beta, dev, canary)
* Firefox (stable, beta, dev, nightly)
* Floorp
* Thorium
* Brave
* Vivaldi
* Librewolf
* Waterfox

# Usage

More examples can be found in the [examples](./examples) directory. Try them with `cargo run --example <example>`!

## Displaying a remote website
```rust
use crowser::{error::CrowserError, RemoteConfig, Window};

fn main() -> Result<(), CrowserError> {
  // Profile directories are specified by you, so put it wherever makes sense!
  let mut profile_dir = PathBuf::from("/path/to/your/app/profiles");

  let config = RemoteConfig {
    url: "https://example.com".to_string(),
  };

  let mut window = Window::new(config, None, profile_dir)?;

  // Make sure the profile is brand-new before launch
  window.clear_profile()?;

  // This will spawn the window and block until it is closed
  window.create()?;

  Ok(())
}
```

## Embedding a local website
```rust
use crowser::{error::CrowserError, include_dir, LocalConfig, Window};

fn main() -> Result<(), CrowserError> {
  let mut profile_dir = PathBuf::from("/path/to/your/app/profiles");

  // include_dir is a re-export of the include_dir crate
  let dir = include_dir::include_dir!("/path/to/your/app/dist");

  //  To be safe, we'll try to find an open port between 8000 and 8050
  for port in 8000..8050 {
    let config = LocalConfig {
      port,
      directory: dir.clone(),
    };

    let mut window = Window::new(config, None, profile_dir.clone())?;

    window.clear_profile()?;

    // Since we're looping, we'll break when we successfully create the window. This will
    // actually block the thread until the window is closed.
    match window.create() {
      Ok(_) => {
        println!("Window created on port {}", port);
        break;
      }
      Err(e) => {
        println!("Error creating window on port {}: {:?}", port, e);
      }
    }
  }

  Ok(())
}
```