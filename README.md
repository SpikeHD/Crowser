<div align="center">
  <h1>Crowser</h1>
  <p>Create "desktop apps" using user-supplied browsers</p>
</div>

<div align="center">
  <img src="https://img.shields.io/github/actions/workflow/status/SpikeHD/crowser/check.yml" />
  <img src="https://img.shields.io/github/repo-size/SpikeHD/crowser" />
  <img src="https://img.shields.io/github/commit-activity/m/SpikeHD/crowser" />
</div>

# Features and Support

## Main features

* Single-digit binary size with minimal dependencies (except for the browser, of course!)
* Multi-platform. Supports Windows, macOS, and Linux. Even on ARM!
* Two-way IPC.
* Support for both local and remote websites.
* Maximizes performance of whatever browser is chosen, and uses an entirely separate browser profile.

> [!NOTE]
> This library will basically forever and always be intended for low-stakes or experimental use. It's really hard to guarantee working functionality and consistency across a bunch of different browsers, so please keep that in mind! 

## Browser support

All browser support comes with a few small caveats. You may notice small inconsistencies between, say, your app running in Firefox and in Chrome. Many of these are also not properly tested, so if you find any issues, feel free to [submit an issue](https://github.com/SpikeHD/Crowser/issues/new/choose)!

* Chrome/Chromium (stable, beta, dev, canary)
* Edge (stable, beta, dev, canary)
* Firefox (stable, beta, dev, nightly)
* Floorp
* Thorium
* Brave
* Vivaldi
* Librewolf
* Waterfox
* Mercury

Want to see what browsers are detected on your system? Run the `browsers` example with `cargo run --example browsers`!

> [!TIP]
> Not seeing your favorite browser? Feel free to submit an issue!
>
> If you have some programming/Rust experience, it's trivial to add new browsers to the [supported list](./src/browser/mod.rs).

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
  window.clean_profile()?;

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

  let config = LocalConfig {
    directory: dir.clone(),
  };

  let mut window = Window::new(config, None, profile_dir.clone())?;

  window.clean_profile()?;

  window.create()?;

  Ok(())
}
```

## Registering a command
```rust
use crowser::{error::CrowserError, ipc::BrowserIpc, RemoteConfig, Window};

fn main() -> Result<(), CrowserError> {
  let mut profile_dir = std::env::current_dir()?;
  profile_dir.push("example_profiles");

  let config = RemoteConfig {
    url: "https://example.com".to_string(),
  };

  let mut window = Window::new(config, None, profile_dir)?;
  let ipc = window.ipc();

  window.clear_profile().unwrap_or_default();

  std::thread::spawn(move || {
    ipc.block_until_initialized().unwrap_or_default();

    ipc
      .register_command(
        "hello",
        |_| {
          println!("Got hello command");
          Ok(serde_json::json!("Hello from Crowser!"))
        },
      )
      .unwrap_or_default();

    std::thread::sleep(std::time::Duration::from_secs(1));

    // Eval some JS that calls that command
    let result = ipc.eval("window.__CROWSER.ipc.invoke('hello')").unwrap_or_default();
    println!("Result: {:?}", result);
  });

  window.create()?;

  Ok(())
}
```

# How does it work?

On a high level, Crowser works by first detecting browser installations on the user's system (using known paths and ~~registry keys~~). Then, depending on the browser chosen, it will make some specific changes to the browser's CLI arguments,
profile directory, or both. For example, for Firefox there is a `user.js` file in all profiles that can control much of the browser's default behavior. In Chromium-based browsers, there are a stupid amount of command-line arguments that can be
used to control the browser's behavior ([check out this huge list!](https://peter.sh/experiments/chromium-command-line-switches/)).

IPC is facilitated through the [Chrome DevTools Protocol](https://chromedevtools.github.io/devtools-protocol/). To keep the binary size small, the implementation is custom and therefore a little scuffed, but developers do not have to
care about it anyways!

# Contributing

Issues, PRs, etc. are all welcome!