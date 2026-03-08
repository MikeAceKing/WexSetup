#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Json, State,
    },
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    env,
    process::Stdio,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{api::shell, AppHandle, Manager, WindowBuilder, WindowUrl};
use tokio::{net::TcpListener, process::Command};

const BRIDGE_SCRIPT: &str = r#"
(() => {
  const invoke = (command, payload = {}) => {
    if (window.__TAURI__?.invoke) {
      return window.__TAURI__.invoke(command, payload);
    }
    return Promise.reject(new Error('Tauri bridge unavailable'));
  };

  const openExternal = (url) => invoke('open_external_url', { url });
  const openBrowserWindow = (url, title) => invoke('open_wexsearch_window', { url, title });
  const minimizeWindow = () => invoke('minimize_current_window');
  const exitToDesktop = () => invoke('exit_to_desktop');

  window.wexio = Object.assign({}, window.wexio || {}, {
    runtime: 'tauri',
    platform: 'desktop',
    desktopBridge: {
      openExternal,
      openBrowserWindow,
      minimizeWindow,
      exitToDesktop,
      localBridgePort: 47821,
    },
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
})();
"#;

#[derive(Clone)]
struct BridgeState {
    token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LaunchAppPayload {
    command: String,
    #[serde(default)]
    args: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct OpenUrlPayload {
    url: String,
}

#[derive(Debug, Deserialize)]
struct OpenFilePayload {
    path: String,
}

#[derive(Debug, Deserialize)]
struct ExecuteCommandPayload {
    command: String,
    #[serde(default)]
    args: Vec<String>,
}

#[derive(Debug, Serialize)]
struct SystemInfoResponse {
    os: String,
    hostname: String,
    username: String,
    installed_apps: Vec<String>,
}

#[derive(Debug, Serialize)]
struct CommandResultResponse {
    command: String,
    code: i32,
    stdout: String,
    stderr: String,
}

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
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
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
            let state = Arc::new(BridgeState {
                token: env::var("WEXSEARCH_BRIDGE_TOKEN")
                    .ok()
                    .filter(|value| !value.trim().is_empty()),
            });

            tauri::async_runtime::spawn(async move {
                if let Err(error) = start_bridge_server(state).await {
                    eprintln!("[wexio-desktop] local bridge error: {error}");
                }
            });

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

async fn start_bridge_server(state: Arc<BridgeState>) -> anyhow::Result<()> {
    let router = Router::new()
        .route("/health", get(health))
        .route("/system/info", get(system_info))
        .route("/launch/app", post(launch_app))
        .route("/open/url", post(open_url))
        .route("/open/file", post(open_file))
        .route("/command/execute", post(execute_command))
        .route("/ws", get(websocket))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:47821").await?;
    axum::serve(listener, router).await?;
    Ok(())
}

async fn health(
    State(state): State<Arc<BridgeState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, Response> {
    validate_request(&headers, &state)?;
    Ok(Json(json!({ "ok": true, "app": "wexio-desktop" })))
}

async fn system_info(
    State(state): State<Arc<BridgeState>>,
    headers: HeaderMap,
) -> Result<Json<SystemInfoResponse>, Response> {
    validate_request(&headers, &state)?;
    Ok(Json(SystemInfoResponse {
        os: env::consts::OS.to_string(),
        hostname: whoami::fallible::hostname().unwrap_or_else(|_| "unknown-host".to_string()),
        username: whoami::username(),
        installed_apps: allowed_launch_commands()
            .iter()
            .map(|command| command.to_string())
            .collect(),
    }))
}

async fn launch_app(
    State(state): State<Arc<BridgeState>>,
    headers: HeaderMap,
    Json(payload): Json<LaunchAppPayload>,
) -> impl IntoResponse {
    if let Err(response) = validate_request(&headers, &state) {
        return response;
    }

    let Some((program, prefix_args)) = map_launch_command(&payload.command) else {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Command not allowed" })),
        )
            .into_response();
    };

    let mut command = Command::new(program);
    command
        .args(prefix_args)
        .args(payload.args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    match command.spawn() {
        Ok(_) => Json(json!({ "ok": true })).into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": error.to_string() })),
        )
            .into_response(),
    }
}

async fn open_url(
    State(state): State<Arc<BridgeState>>,
    headers: HeaderMap,
    Json(payload): Json<OpenUrlPayload>,
) -> impl IntoResponse {
    if let Err(response) = validate_request(&headers, &state) {
        return response;
    }

    match open_target(&payload.url).await {
        Ok(_) => Json(json!({ "ok": true })).into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": error })),
        )
            .into_response(),
    }
}

async fn open_file(
    State(state): State<Arc<BridgeState>>,
    headers: HeaderMap,
    Json(payload): Json<OpenFilePayload>,
) -> impl IntoResponse {
    if let Err(response) = validate_request(&headers, &state) {
        return response;
    }

    match open_target(&payload.path).await {
        Ok(_) => Json(json!({ "ok": true })).into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": error })),
        )
            .into_response(),
    }
}

async fn execute_command(
    State(state): State<Arc<BridgeState>>,
    headers: HeaderMap,
    Json(payload): Json<ExecuteCommandPayload>,
) -> impl IntoResponse {
    if let Err(response) = validate_request(&headers, &state) {
        return response;
    }

    let Some((program, prefix_args)) = map_system_command(&payload.command) else {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Command not allowed" })),
        )
            .into_response();
    };

    let output = Command::new(program)
        .args(prefix_args)
        .args(payload.args)
        .output()
        .await;

    match output {
        Ok(result) => Json(CommandResultResponse {
            command: payload.command,
            code: result.status.code().unwrap_or_default(),
            stdout: String::from_utf8_lossy(&result.stdout).to_string(),
            stderr: String::from_utf8_lossy(&result.stderr).to_string(),
        })
        .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": error.to_string() })),
        )
            .into_response(),
    }
}

async fn websocket(
    ws: WebSocketUpgrade,
    State(state): State<Arc<BridgeState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(response) = validate_request(&headers, &state) {
        return response;
    }

    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    let _ = socket
        .send(Message::Text(
            "{\"ok\":true,\"bridge\":\"wexio-desktop\",\"port\":47821}".into(),
        ))
        .await;
}

fn allowed_launch_commands() -> &'static [&'static str] {
    &["code", "chrome", "firefox", "edge", "terminal"]
}

fn is_allowed_origin(origin: &str) -> bool {
    let normalized = origin.trim().to_lowercase();
    (normalized.starts_with("https://")
        && (normalized.contains(".wexio.") || normalized.ends_with(".wexio.be")))
        || normalized.starts_with("http://localhost")
        || normalized.starts_with("http://127.0.0.1")
}

fn validate_request(headers: &HeaderMap, state: &BridgeState) -> Result<(), axum::response::Response> {
    let origin = headers
        .get("x-wexio-origin")
        .or_else(|| headers.get("origin"))
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();

    if !is_allowed_origin(origin) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Origin not allowed" })),
        )
            .into_response());
    }

    if let Some(token) = &state.token {
        let provided = headers
            .get("x-wexsearch-token")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default();

        if provided != token {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Invalid bridge token" })),
            )
                .into_response());
        }
    }

    Ok(())
}

fn map_launch_command(command: &str) -> Option<(&'static str, Vec<&'static str>)> {
    match (env::consts::OS, command) {
        ("windows", "code") => Some(("code", vec![])),
        ("windows", "chrome") => Some(("chrome", vec![])),
        ("windows", "firefox") => Some(("firefox", vec![])),
        ("windows", "edge") => Some(("msedge", vec![])),
        ("windows", "terminal") => Some(("cmd", vec![])),
        ("macos", "code") => Some(("open", vec!["-a", "Visual Studio Code"])),
        ("macos", "chrome") => Some(("open", vec!["-a", "Google Chrome"])),
        ("macos", "firefox") => Some(("open", vec!["-a", "Firefox"])),
        ("macos", "edge") => Some(("open", vec!["-a", "Microsoft Edge"])),
        ("macos", "terminal") => Some(("open", vec!["-a", "Terminal"])),
        (_, "code") => Some(("code", vec![])),
        (_, "chrome") => Some(("google-chrome", vec![])),
        (_, "firefox") => Some(("firefox", vec![])),
        (_, "edge") => Some(("microsoft-edge", vec![])),
        (_, "terminal") => Some(("x-terminal-emulator", vec![])),
        _ => None,
    }
}

fn map_system_command(command: &str) -> Option<(&'static str, Vec<&'static str>)> {
    match (env::consts::OS, command) {
        ("windows", "date") => Some(("powershell", vec!["-NoProfile", "-Command", "Get-Date"])),
        ("windows", "whoami") => Some(("whoami", vec![])),
        ("windows", "pwd") => Some(("powershell", vec!["-NoProfile", "-Command", "Get-Location"])),
        ("windows", "uname") => Some(("cmd", vec!["/C", "ver"])),
        (_, "date") => Some(("date", vec![])),
        (_, "whoami") => Some(("whoami", vec![])),
        (_, "pwd") => Some(("pwd", vec![])),
        (_, "uname") => Some(("uname", vec!["-a"])),
        _ => None,
    }
}

async fn open_target(target: &str) -> Result<(), String> {
    let mut command = if env::consts::OS == "windows" {
        let mut command = Command::new("cmd");
        command.args(["/C", "start", "", target]);
        command
    } else if env::consts::OS == "macos" {
        let mut command = Command::new("open");
        command.arg(target);
        command
    } else {
        let mut command = Command::new("xdg-open");
        command.arg(target);
        command
    };

    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    command.spawn().map(|_| ()).map_err(|error| error.to_string())
}
