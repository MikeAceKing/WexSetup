#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{api::shell, AppHandle, Manager, WindowBuilder, WindowUrl};

const BRIDGE_SCRIPT: &str = r#"
(() => {
  const resolveInvoke = () => {
    if (typeof window === 'undefined') {
      return null;
    }

    const tauriGlobal = window.__TAURI__;
    if (!tauriGlobal) {
      return null;
    }

    if (typeof tauriGlobal.invoke === 'function') {
      return tauriGlobal.invoke.bind(tauriGlobal);
    }

    if (typeof tauriGlobal.tauri?.invoke === 'function') {
      return tauriGlobal.tauri.invoke.bind(tauriGlobal.tauri);
    }

    if (typeof tauriGlobal.core?.invoke === 'function') {
      return tauriGlobal.core.invoke.bind(tauriGlobal.core);
    }

    return null;
  };

  const invoke = (command, payload = {}) => {
    const runtimeInvoke = resolveInvoke();
    if (runtimeInvoke) {
      return runtimeInvoke(command, payload);
    }
    return Promise.reject(new Error('Tauri bridge unavailable'));
  };

  const openExternal = (url) => invoke('open_external_url', { url });
  const openBrowserWindow = (url, title) => invoke('open_wexsearch_window', { url, title });
  const minimizeWindow = () => invoke('minimize_current_window');
  const maximizeWindow = () => invoke('maximize_current_window');
  const closeWindow = () => invoke('close_current_window');
  const exitToDesktop = () => invoke('exit_to_desktop');

  window.wexio = Object.assign({}, window.wexio || {}, {
    runtime: 'tauri',
    platform: 'desktop',
    desktopBridge: {
      openExternal,
      openBrowserWindow,
      minimizeWindow,
      maximizeWindow,
      closeWindow,
      exitToDesktop,
    },
    openExternal,
    openBrowserWindow,
    minimizeWindow,
    maximizeWindow,
    closeWindow,
    exitToDesktop,
  });

  const isHttpUrl = (value) => /^https?:\/\//i.test(String(value || '').trim());
  const isExternalScheme = (value) => /^(mailto:|tel:)/i.test(String(value || '').trim());
  const originalOpen = typeof window.open === 'function' ? window.open.bind(window) : null;

  window.open = function patchedOpen(url, target, features) {
    const nextUrl = String(url || '').trim();
    if (!nextUrl) {
      return originalOpen ? originalOpen(url, target, features) : null;
    }

    if (isHttpUrl(nextUrl)) {
      openBrowserWindow(nextUrl, nextUrl);
      return null;
    }

    if (isExternalScheme(nextUrl)) {
      openExternal(nextUrl);
      return null;
    }

    return originalOpen ? originalOpen(url, target, features) : null;
  };

  document.addEventListener(
    'click',
    (event) => {
      const path = event.composedPath ? event.composedPath() : [];
      const anchor = path.find((node) => node instanceof HTMLAnchorElement && node.href);
      if (!anchor) return;

      const href = String(anchor.href || '').trim();
      if (!href) return;

      const wantsNewWindow =
        anchor.target === '_blank' ||
        anchor.hasAttribute('download') ||
        anchor.dataset?.wexioDesktopOpen === 'new-window';

      if (isHttpUrl(href) && wantsNewWindow) {
        event.preventDefault();
        event.stopPropagation();
        openBrowserWindow(href, anchor.textContent?.trim() || href);
        return;
      }

      if (isExternalScheme(href)) {
        event.preventDefault();
        event.stopPropagation();
        openExternal(href);
      }
    },
    true
  );
})();
"#;

#[tauri::command]
fn open_external_url(app: AppHandle, url: String) -> Result<(), String> {
    shell::open(&app.shell_scope(), url, None).map_err(|error| error.to_string())
}

#[tauri::command]
fn open_wexsearch_window(app: AppHandle, url: String, title: Option<String>) -> Result<(), String> {
    let parsed_url: url::Url = url
        .parse()
        .map_err(|error: url::ParseError| error.to_string())?;

    let label = format!(
        "wexsearch-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_millis()
    );

    let window = WindowBuilder::new(&app, label, WindowUrl::External(parsed_url))
        .title(title.unwrap_or_else(|| "Wexio Browser".to_string()))
        .inner_size(1280.0, 800.0)
        .resizable(true)
        .visible(true)
        .focused(true)
        .center()
        .build()
        .map_err(|error| error.to_string())?;

    let _ = window.set_focus();
    Ok(())
}

#[tauri::command]
fn minimize_current_window(window: tauri::Window) -> Result<(), String> {
    window.minimize().map_err(|error| error.to_string())
}

#[tauri::command]
fn maximize_current_window(window: tauri::Window) -> Result<(), String> {
    let is_maximized = window.is_maximized().map_err(|error| error.to_string())?;
    if is_maximized {
        window.unmaximize().map_err(|error| error.to_string())
    } else {
        window.maximize().map_err(|error| error.to_string())
    }
}

#[tauri::command]
fn close_current_window(window: tauri::Window) -> Result<(), String> {
    window.close().map_err(|error| error.to_string())
}

#[tauri::command]
fn exit_to_desktop(app: AppHandle, window: tauri::Window) -> Result<(), String> {
    if window.label() == "main" {
        window.minimize().map_err(|error| error.to_string())?;
        return Ok(());
    }

    if let Some(main_window) = app.get_window("main") {
        let _ = main_window.show();
        let _ = main_window.set_focus();
    }

    window.close().map_err(|error| error.to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            open_external_url,
            open_wexsearch_window,
            minimize_current_window,
            maximize_current_window,
            close_current_window,
            exit_to_desktop
        ])
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
