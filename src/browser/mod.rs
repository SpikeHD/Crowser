use std::{env, path::PathBuf};

pub mod chromium;
pub mod firefox;

trait Append<T> {
  fn append(self, value: T) -> Self;
}

impl<T> Append<T> for Vec<T> {
  fn append(mut self, value: T) -> Self {
    self.push(value);
    self
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserKind {
  Chromium,
  Gecko,
  WebKit,
  Unknown,
}

#[derive(Debug, Clone)]
pub struct BrowserWindowsConfig {
  pub paths: Vec<PathBuf>,
  pub registry_keys: Vec<&'static str>,
}

#[derive(Debug, Clone)]
pub struct Browser {
  pub name: &'static str,
  pub kind: BrowserKind,

  /// Contains several possible paths as well as registry keys in order to check browser installation location
  pub win: BrowserWindowsConfig,

  /// Contains possible binary names for the browser
  pub unix: Vec<&'static str>,

  /// Contains possible paths for the browser
  pub mac: Vec<PathBuf>,
}

lazy_static::lazy_static! {
  static ref BROWSERS: Vec<Browser> = vec![
    Browser {
      name: "chrome",
      kind: BrowserKind::Chromium,
      win: BrowserWindowsConfig {
        paths: generate_windows_paths(
          PathBuf::from("Google\\Chrome\\Application\\chrome.exe")
        ).append(
          PathBuf::from(
            format!("{}\\scoop\\apps\\googlechrome\\current\\chrome.exe", env::var("USERPROFILE").unwrap_or_default())
          )
        ),
        registry_keys: vec![],
      },
      unix: vec!["chrome", "google-chrome", "chrome-browser", "google-chrome-stable"],
      mac: vec![PathBuf::from("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome")],
    },
    Browser {
      name: "chrome_beta",
      kind: BrowserKind::Chromium,
      win: BrowserWindowsConfig {
        paths: generate_windows_paths(PathBuf::from("Google\\Chrome Beta\\Application\\chrome.exe")),
        registry_keys: vec![],
      },
      unix: vec!["chrome-beta", "google-chrome-beta", "chrome-beta-browser", "chrome-browser-beta"],
      mac: vec![PathBuf::from("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome Beta")],
    },
    Browser {
      name: "chrome_dev",
      kind: BrowserKind::Chromium,
      win: BrowserWindowsConfig {
        paths: generate_windows_paths(PathBuf::from("Google\\Chrome Dev\\Application\\chrome.exe")),
        registry_keys: vec![],
      },
      unix: vec!["chrome-unstable", "google-chrome-unstable", "chrome-unstable-browser", "chrome-browser-unstable"],
      mac: vec![PathBuf::from("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome Dev")],
    },
    Browser {
      name: "chrome_canary",
      kind: BrowserKind::Chromium,
      win: BrowserWindowsConfig {
        paths: generate_windows_paths(PathBuf::from("Google\\Chrome SxS\\Application\\chrome.exe")),
        registry_keys: vec![],
      },
      unix: vec!["chrome-canary", "google-chrome-canary", "chrome-canary-browser", "chrome-browser-canary"],
      mac: vec![PathBuf::from("/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary")],
    },
    Browser {
      name: "chromium",
      kind: BrowserKind::Chromium,
      win: BrowserWindowsConfig {
        paths: generate_windows_paths(
          PathBuf::from("Chromium\\Application\\chrome.exe")
        ).append(
          PathBuf::from(
            format!("{}\\scoop\\apps\\chromium\\current\\chrome.exe", env::var("USERPROFILE").unwrap_or_default())
          )
        ),
        registry_keys: vec![],
      },
      unix: vec!["chromium", "chromium-browser"],
      mac: vec![PathBuf::from("/Applications/Chromium.app/Contents/MacOS/Chromium")],
    },
    Browser {
      name: "edge",
      kind: BrowserKind::Chromium,
      win: BrowserWindowsConfig {
        paths: generate_windows_paths(PathBuf::from("Microsoft\\Edge\\Application\\msedge.exe")),
        registry_keys: vec![],
      },
      unix: vec!["microsoft-edge", "microsoft-edge-stable", "microsoft-edge-browser"],
      mac: vec![PathBuf::from("/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge")],
    },
    Browser {
      name: "edge_beta",
      kind: BrowserKind::Chromium,
      win: BrowserWindowsConfig {
        paths: generate_windows_paths(PathBuf::from("Microsoft\\Edge Beta\\Application\\msedge.exe")),
        registry_keys: vec![],
      },
      unix: vec!["microsoft-edge-beta", "microsoft-edge-browser-beta", "microsoft-edge-beta-browser"],
      mac: vec![PathBuf::from("/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge Beta")],
    },
    Browser {
      name: "edge_dev",
      kind: BrowserKind::Chromium,
      win: BrowserWindowsConfig {
        paths: generate_windows_paths(PathBuf::from("Microsoft\\Edge Dev\\Application\\msedge.exe")),
        registry_keys: vec![],
      },
      unix: vec!["microsoft-edge-dev", "microsoft-edge-browser-dev", "microsoft-edge-dev-browser"],
      mac: vec![PathBuf::from("/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge Dev")],
    },
    Browser {
      name: "edge_canary",
      kind: BrowserKind::Chromium,
      win: BrowserWindowsConfig {
        paths: generate_windows_paths(PathBuf::from("Microsoft\\Edge SxS\\Application\\msedge.exe")),
        registry_keys: vec![],
      },
      unix: vec!["microsoft-edge-canary", "microsoft-edge-browser-canary", "microsoft-edge-browser-canary"],
      mac: vec![PathBuf::from("/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge Canary")],
    },
    Browser {
      name: "thorium",
      kind: BrowserKind::Chromium,
      win: BrowserWindowsConfig {
        paths: generate_windows_paths(PathBuf::from("Thorium\\Application\\thorium.exe")),
        registry_keys: vec![],
      },
      unix: vec!["thorium", "thorium-browser"],
      mac: vec![PathBuf::from("/Applications/Thorium.app/Contents/MacOS/Thorium")],
    },
    Browser {
      name: "brave",
      kind: BrowserKind::Chromium,
      win: BrowserWindowsConfig {
        paths: generate_windows_paths(PathBuf::from("BraveSoftware\\Brave-Browser\\Application\\brave.exe")),
        registry_keys: vec![],
      },
      unix: vec!["brave", "brave-browser"],
      mac: vec![PathBuf::from("/Applications/Brave Browser.app/Contents/MacOS/Brave Browser")],
    },
    Browser {
      name: "vivaldi",
      kind: BrowserKind::Chromium,
      win: BrowserWindowsConfig {
        paths: generate_windows_paths(PathBuf::from("Vivaldi\\Application\\vivaldi.exe")),
        registry_keys: vec![],
      },
      unix: vec!["vivaldi", "vivaldi-browser"],
      mac: vec![PathBuf::from("/Applications/Vivaldi.app/Contents/MacOS/Vivaldi")],
    },
    Browser {
      name: "firefox",
      kind: BrowserKind::Gecko,
      win: BrowserWindowsConfig {
        paths: generate_windows_paths(
          PathBuf::from("Mozilla Firefox\\firefox.exe")
        ).append(
          PathBuf::from(
            format!("{}\\scoop\\apps\\firefox\\current\\firefox.exe", env::var("USERPROFILE").unwrap_or_default())
          )
        ),
        registry_keys: vec![],
      },
      unix: vec!["firefox", "firefox-browser"],
      mac: vec![PathBuf::from("/Applications/Firefox.app/Contents/MacOS/firefox")],
    },
    Browser {
      name: "firefox_developer",
      kind: BrowserKind::Gecko,
      win: BrowserWindowsConfig {
        paths: generate_windows_paths(PathBuf::from("Mozilla Firefox Developer Edition\\firefox.exe")),
        registry_keys: vec![],
      },
      unix: vec![], // No specific unix binary names for Firefox Developer Edition
      mac: vec![], // No specific macOS path for Firefox Developer Edition
    },
    Browser {
      name: "firefox_nightly",
      kind: BrowserKind::Gecko,
      win: BrowserWindowsConfig {
        paths: generate_windows_paths(PathBuf::from("Mozilla Firefox Nightly\\firefox.exe")),
        registry_keys: vec![],
      },
      unix: vec!["firefox-nightly", "firefox-nightly-browser", "firefox-browser-nightly"],
      mac: vec![PathBuf::from("/Applications/Firefox Nightly.app/Contents/MacOS/firefox")],
    },
    Browser {
      name: "floorp",
      kind: BrowserKind::Gecko,
      win: BrowserWindowsConfig {
        paths: generate_windows_paths(PathBuf::from("Ablaze Floorp\\floorp.exe")),
        registry_keys: vec![],
      },
      unix: vec!["floorp", "floorp-browser"],
      mac: vec![PathBuf::from("/Applications/Floorp.app/Contents/MacOS/floorp")],
    },
    Browser {
      name: "librewolf",
      kind: BrowserKind::Gecko,
      win: BrowserWindowsConfig {
        paths: generate_windows_paths(PathBuf::from("LibreWolf\\librewolf.exe")),
        registry_keys: vec![],
      },
      unix: vec!["librewolf", "librewolf-browser"],
      mac: vec![PathBuf::from("/Applications/LibreWolf.app/Contents/MacOS/librewolf")],
    },
    Browser {
      name: "waterfox",
      kind: BrowserKind::Gecko,
      win: BrowserWindowsConfig {
        paths: generate_windows_paths(PathBuf::from("Waterfox\\waterfox.exe")),
        registry_keys: vec![],
      },
      unix: vec!["waterfox", "waterfox-browser"],
      mac: vec![PathBuf::from("/Applications/Waterfox.app/Contents/MacOS/waterfox")],
    },
  ];
}

fn generate_windows_paths(path: PathBuf) -> Vec<PathBuf> {
  // Create a vec of all possibilities for installation location (program files, program files(x86), etc)
  let mut paths = vec![];
  let prefixes = vec![
    env::var("PROGRAMFILES").unwrap_or_default(),
    env::var("PROGRAMFILES(X86)").unwrap_or_default(),
    env::var("LOCALAPPDATA").unwrap_or_default(),
  ];

  for prefix in prefixes {
    let mut full_path = PathBuf::from(prefix);
    full_path.push(path.clone());
    paths.push(full_path);
  }

  paths
}

/// Get all supported browsers
pub fn get_supported_browsers() -> Vec<Browser> {
  BROWSERS.iter().cloned().collect()
}

/// Get the best browser based on the provided kind.
/// If no kind is provided, the first found supported browser is returned.
pub fn get_best_browser(kind: Option<BrowserKind>) -> Option<(Browser, PathBuf)> {
  let browsers = get_all_existing_browsers();

  for (browser, path) in browsers {
    if let Some(k) = kind {
      if browser.kind == k {
        return Some((browser, path));
      }
    } else {
      return Some((browser, path));
    }
  }

  None
}

/// Get all browsers available on the system
pub fn get_all_existing_browsers() -> Vec<(Browser, PathBuf)> {
  let mut valid: Vec<(Browser, PathBuf)> = vec![];

  // Now look for the first browser that actually exists on the system
  #[cfg(target_os = "windows")]
  for browser in BROWSERS.iter() {
    for path in &browser.win.paths {
      if path.exists() {
        valid.push((browser.clone(), path.clone()));
      }
    }
  }

  #[cfg(target_os = "linux")]
  for browser in BROWSERS.iter() {
    for binary in &browser.unix {
      if which::which(binary).is_ok() {
        valid.push((browser.clone(), PathBuf::from(binary)));
      }
    }
  }

  #[cfg(target_os = "macos")]
  for browser in BROWSERS.iter() {
    for path in &browser.mac {
      if path.exists() {
        valid.push((browser.clone(), path.clone()));
      }
    }
  }

  valid
}
