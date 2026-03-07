#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager, WindowBuilder, WindowUrl};

const BRIDGE_SCRIPT: &str = r#"
(() => {
  const CONTROL_BAR_ID = '__wexio_desktop_controls__';

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

  const minimizeWindow = () => {
    if (window.__TAURI__?.invoke) {
      return window.__TAURI__.invoke('minimize_current_window');
    }
    return Promise.reject(new Error('Tauri bridge unavailable'));
  };

  const exitToDesktop = () => {
    if (window.__TAURI__?.invoke) {
      return window.__TAURI__.invoke('exit_to_desktop');
    }
    return Promise.reject(new Error('Tauri bridge unavailable'));
  };

  window.wexio = Object.assign({}, window.wexio || {}, {
    runtime: 'tauri',
    platform: 'desktop',
    openExternal,
    openBrowserWindow,
    minimizeWindow,
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

  const injectControls = () => {
    if (!document.body || document.getElementById(CONTROL_BAR_ID)) return;

    const style = document.createElement('style');
    style.textContent = `
      #${CONTROL_BAR_ID} {
        position: fixed;
        top: 14px;
        right: 14px;
        z-index: 2147483647;
        display: flex;
        gap: 8px;
        align-items: center;
        padding: 8px 10px;
        border-radius: 999px;
        background: rgba(11, 18, 32, 0.86);
        border: 1px solid rgba(148, 163, 184, 0.28);
        backdrop-filter: blur(14px);
        box-shadow: 0 10px 30px rgba(0, 0, 0, 0.28);
        font-family: system-ui, sans-serif;
      }

      #${CONTROL_BAR_ID} button {
        appearance: none;
        border: 0;
        border-radius: 999px;
        background: rgba(30, 41, 59, 0.92);
        color: #e2e8f0;
        padding: 8px 12px;
        font-size: 12px;
        font-weight: 700;
        line-height: 1;
        cursor: pointer;
      }

      #${CONTROL_BAR_ID} button:hover {
        background: rgba(37, 99, 235, 0.95);
        color: white;
      }
    `;

    const controls = document.createElement('div');
    controls.id = CONTROL_BAR_ID;
    controls.setAttribute('data-wexio-desktop-controls', 'true');
    controls.innerHTML = `
      <button type="button" data-action="desktop">Desktop</button>
      <button type="button" data-action="minimize">Minimize</button>
    `;

    controls.addEventListener('click', (event) => {
      const button = event.target instanceof HTMLElement ? event.target.closest('button') : null;
      const action = button?.getAttribute('data-action');
      if (action === 'desktop') {
        exitToDesktop();
      }
      if (action === 'minimize') {
        minimizeWindow();
      }
    });

    document.head.appendChild(style);
    document.body.appendChild(controls);
  };

  const bootControls = () => {
    if (document.readyState === 'loading') {
      document.addEventListener('DOMContentLoaded', injectControls, { once: true });
      return;
    }
    injectControls();
  };

  bootControls();
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
        .visible(true)
        .build()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn minimize_current_window(app: AppHandle, window: tauri::Window) -> Result<(), String> {
    window.minimize().map_err(|error| error.to_string())?;

    if let Some(main_window) = app.get_window("main") {
        let _ = main_window.show();
    }

    Ok(())
}

#[tauri::command]
fn exit_to_desktop(app: AppHandle, window: tauri::Window) -> Result<(), String> {
    if window.label() == "main" {
        window
            .eval("window.location.replace('https://ui.wexio.be/platform');")
            .map_err(|error| error.to_string())?;
        return Ok(());
    }

    if let Some(main_window) = app.get_window("main") {
        let _ = main_window.show();
        let _ = main_window.set_focus();
        let _ = main_window.eval("window.location.replace('https://ui.wexio.be/platform');");
    }

    window.close().map_err(|error| error.to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            open_external_url,
            open_wexsearch_window,
            minimize_current_window,
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
