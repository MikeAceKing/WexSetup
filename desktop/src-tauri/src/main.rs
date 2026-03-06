#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager, WindowBuilder, WindowUrl};

const BRIDGE_SCRIPT: &str = r#"
(() => {
  const openExternal = (url) => {
    if (window.__TAURI__?.invoke) {
      return window.__TAURI__.invoke('open_external_url', { url });
    }
    return Promise.reject(new Error('Tauri bridge unavailable'));
  };

  const openBrowserWindow = (url, title) => {
    if (window.__TAURI__?.invoke) {
      return window.__TAURI__.invoke('open_wexsearch_window', { url, title });
    }
    return Promise.reject(new Error('Tauri bridge unavailable'));
  };

  window.wexio = Object.assign({}, window.wexio || {}, {
    runtime: 'tauri',
    platform: 'desktop',
    openExternal,
    openBrowserWindow,
  });
})();
"#;

#[tauri::command]
fn open_external_url(app: AppHandle, url: String) -> Result<(), String> {
  tauri::api::shell::open(&app.shell_scope(), url, None).map_err(|error| error.to_string())
}

#[tauri::command]
fn open_wexsearch_window(app: AppHandle, url: String, title: Option<String>) -> Result<(), String> {
  let parsed_url: url::Url = url
    .parse()
    .map_err(|error: url::ParseError| error.to_string())?;

  let label = format!(
    "wexsearch-{}",
    SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .map_err(|error| error.to_string())?
      .as_millis()
  );

  WindowBuilder::new(&app, label, WindowUrl::External(parsed_url))
    .title(title.unwrap_or_else(|| "WexSearch".to_string()))
    .inner_size(1280.0, 800.0)
    .resizable(true)
    .build()
    .map(|_| ())
    .map_err(|error| error.to_string())
}

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![open_external_url, open_wexsearch_window])
    .setup(|app| {
      let main_window = app.get_window("main").ok_or("main window unavailable")?;
      let _ = main_window.eval(BRIDGE_SCRIPT);
      Ok(())
    })
    .on_page_load(|window, _| {
      let _ = window.eval(BRIDGE_SCRIPT);
    })
    .run(tauri::generate_context!())
    .expect("error while running Wexio Desktop");
}
