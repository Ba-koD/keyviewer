use crate::state::{AppState, TargetConfig};
use crate::window_info;
use axum::{
    body::Body,
    extract::{
        ws::{Message, WebSocket},
        State as AxumState, WebSocketUpgrade,
    },
    http::{header, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::get,
    Router,
};
use parking_lot::RwLock;
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tower_http::cors::{Any, CorsLayer};

// Embed UI files at compile time
const OVERLAY_HTML: &str = include_str!("../../ui/overlay.html");
const OVERLAY_CSS: &str = include_str!("../../ui/overlay.css");
const CONTROL_HTML: &str = include_str!("../../ui/control.html");
const CONTROL_CSS: &str = include_str!("../../ui/control.css");
const LAUNCHER_CSS: &str = include_str!("../../ui/launcher.css");
const FAVICON: &[u8] = include_bytes!("../../ui/favicon.ico");

type SharedState = Arc<RwLock<AppState>>;

// Server controller for start/stop
pub struct ServerController {
    runtime: tokio::runtime::Runtime,
    handle: Option<tokio::task::AbortHandle>,
    running: Arc<parking_lot::Mutex<bool>>,
}

impl ServerController {
    pub fn new() -> Self {
        Self {
            runtime: tokio::runtime::Runtime::new().unwrap(),
            handle: None,
            running: Arc::new(parking_lot::Mutex::new(false)),
        }
    }

    pub fn start(&mut self, state: SharedState, port: u16) -> Result<(), String> {
        let mut running = self.running.lock();
        if *running {
            return Err("서버가 이미 실행 중입니다".to_string());
        }

        // Check if port is available before starting
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        match std::net::TcpListener::bind(addr) {
            Ok(_) => {
                // Port is available, proceed
            }
            Err(e) => {
                // Port is in use or other bind error
                let error_msg = if e.kind() == std::io::ErrorKind::AddrInUse {
                    format!("포트 {}가 이미 사용 중입니다. 다른 프로그램이 해당 포트를 사용하고 있는지 확인해주세요.", port)
                } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                    format!("포트 {}에 대한 접근 권한이 없습니다. 관리자 권한으로 실행하거나 1024 이상의 포트를 사용해주세요.", port)
                } else {
                    format!("포트 {}를 바인드할 수 없습니다: {}", port, e)
                };
                return Err(error_msg);
            }
        }

        let running_clone = self.running.clone();
        let handle = self.runtime.spawn(async move {
            let app = create_router(state);
            let addr = SocketAddr::from(([127, 0, 0, 1], port));
            
            println!("Server listening on http://{}", addr);
            *running_clone.lock() = true;

            match tokio::net::TcpListener::bind(addr).await {
                Ok(listener) => {
                    if let Err(e) = axum::serve(listener, app).await {
                        eprintln!("Server error: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to bind to port {}: {}", port, e);
                }
            }

            *running_clone.lock() = false;
        });

        self.handle = Some(handle.abort_handle());
        *running = true;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), String> {
        let mut running = self.running.lock();
        if !*running {
            return Err("서버가 실행 중이 아닙니다".to_string());
        }

        if let Some(handle) = self.handle.take() {
            handle.abort();
        }

        *running = false;
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        *self.running.lock()
    }
}


fn create_router(state: SharedState) -> Router {
    // Setup CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/", get(root_redirect))
        .route("/overlay", get(get_overlay))
        .route("/overlay.html", get(get_overlay))
        .route("/control", get(get_control))
        .route("/control.html", get(get_control))
        .route("/static/overlay.css", get(get_overlay_css))
        .route("/static/control.css", get(get_control_css))
        .route("/static/launcher.css", get(get_launcher_css))
        .route("/static/favicon.ico", get(get_favicon))
        .route("/ws", get(websocket_handler))
        .route("/api/windows", get(api_windows))
        .route("/api/foreground", get(api_foreground))
        .route("/api/target", get(api_get_target))
        .route("/api/target", axum::routing::post(api_set_target))
        .route("/api/config", get(api_get_config))
        .route("/api/config", axum::routing::post(api_set_config))
        .route("/api/overlay-config", get(api_get_overlay_config))
        .route("/api/overlay-config", axum::routing::post(api_set_overlay_config))
        .route("/api/launcher-language", get(api_get_launcher_language))
        .route("/api/focus", axum::routing::post(api_focus_window))
        .layer(cors)
        .with_state(state)
}

async fn root_redirect() -> impl IntoResponse {
    (StatusCode::FOUND, [(header::LOCATION, "/control")])
}

// Serve embedded HTML files with no-cache headers for OBS browser source
async fn get_overlay() -> impl IntoResponse {
    Response::builder()
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
        .header(header::PRAGMA, "no-cache")
        .header(header::EXPIRES, "0")
        .body(Body::from(OVERLAY_HTML))
        .unwrap()
}

async fn get_control() -> impl IntoResponse {
    Response::builder()
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
        .header(header::PRAGMA, "no-cache")
        .header(header::EXPIRES, "0")
        .body(Body::from(CONTROL_HTML))
        .unwrap()
}

// Serve embedded CSS files with no-cache headers for OBS browser source
async fn get_overlay_css() -> impl IntoResponse {
    Response::builder()
        .header(header::CONTENT_TYPE, "text/css; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
        .header(header::PRAGMA, "no-cache")
        .header(header::EXPIRES, "0")
        .body(Body::from(OVERLAY_CSS))
        .unwrap()
}

async fn get_control_css() -> impl IntoResponse {
    Response::builder()
        .header(header::CONTENT_TYPE, "text/css; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
        .header(header::PRAGMA, "no-cache")
        .header(header::EXPIRES, "0")
        .body(Body::from(CONTROL_CSS))
        .unwrap()
}

async fn get_launcher_css() -> impl IntoResponse {
    Response::builder()
        .header(header::CONTENT_TYPE, "text/css; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
        .header(header::PRAGMA, "no-cache")
        .header(header::EXPIRES, "0")
        .body(Body::from(LAUNCHER_CSS))
        .unwrap()
}

// Serve embedded favicon with no-cache headers
async fn get_favicon() -> impl IntoResponse {
    Response::builder()
        .header(header::CONTENT_TYPE, "image/x-icon")
        .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
        .header(header::PRAGMA, "no-cache")
        .header(header::EXPIRES, "0")
        .body(Body::from(FAVICON.to_vec()))
        .unwrap()
}

async fn websocket_handler(ws: WebSocketUpgrade, AxumState(state): AxumState<SharedState>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: SharedState) {
    // Send initial state
    let keys = {
        let state_lock = state.read();
        state_lock.get_keys()
    };
    let initial_msg = json!({ "keys": keys });
    if socket
        .send(Message::Text(initial_msg.to_string()))
        .await
        .is_err()
    {
        return;
    }

    // Keep connection alive and send updates
    loop {
        sleep(Duration::from_millis(100)).await;

        let keys = {
            let state_lock = state.read();
            state_lock.get_keys()
        };
        let msg = json!({ "keys": keys });

        if socket.send(Message::Text(msg.to_string())).await.is_err() {
            break;
        }
    }
}

async fn api_windows() -> impl IntoResponse {
    let windows = window_info::get_all_windows();
    Json(windows)
}

async fn api_foreground() -> impl IntoResponse {
    if let Some(window) = window_info::get_foreground_window() {
        Json(json!({
            "hwnd": window.hwnd,
            "title": window.title,
            "process_name": window.process,
            "class": window.class,
        }))
    } else {
        Json(json!({
            "hwnd": null,
            "title": null,
            "process_name": null,
            "class": null,
        }))
    }
}

async fn api_get_target(AxumState(state): AxumState<SharedState>) -> impl IntoResponse {
    let state_lock = state.read();
    Json(json!({
        "mode": state_lock.target_config.mode,
        "value": state_lock.target_config.value,
    }))
}

async fn api_set_target(
    AxumState(state): AxumState<SharedState>,
    Json(payload): Json<TargetConfig>,
) -> impl IntoResponse {
    let mut state_lock = state.write();
    state_lock.target_config = payload.clone();
    state_lock.clear_keys();
    
    // Save to registry
    let _ = crate::settings::save_target_config(
        &payload.mode,
        payload.value.as_deref(),
    );
    
    Json(json!({
        "mode": payload.mode,
        "value": payload.value,
    }))
}

async fn api_get_config(AxumState(state): AxumState<SharedState>) -> impl IntoResponse {
    let state_lock = state.read();
    Json(json!({
        "port": state_lock.app_config.port,
    }))
}

async fn api_set_config(
    AxumState(state): AxumState<SharedState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some(port) = payload.get("port").and_then(|p| p.as_u64()) {
        let port = port as u16;
        if (1000..=65535).contains(&port) {
            let mut state_lock = state.write();
            state_lock.app_config.port = port;
            return Json(json!({
                "ok": true,
                "message": "Saved. Restart server to apply.",
                "port": port,
            }));
        }
    }
    Json(json!({
        "ok": false,
        "message": "Port must be between 1000-65535",
    }))
}

async fn api_get_overlay_config(AxumState(state): AxumState<SharedState>) -> impl IntoResponse {
    let state_lock = state.read();
    Json(state_lock.app_config.overlay.clone())
}

async fn api_set_overlay_config(
    AxumState(state): AxumState<SharedState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let mut state_lock = state.write();
    let overlay = &mut state_lock.app_config.overlay;

    if let Some(v) = payload.get("fade_in_ms").and_then(|x| x.as_u64()) {
        overlay.fade_in_ms = v as u32;
    }
    if let Some(v) = payload.get("fade_out_ms").and_then(|x| x.as_u64()) {
        overlay.fade_out_ms = v as u32;
    }
    if let Some(v) = payload.get("chip_bg").and_then(|x| x.as_str()) {
        overlay.chip_bg = v.to_string();
    }
    if let Some(v) = payload.get("chip_fg").and_then(|x| x.as_str()) {
        overlay.chip_fg = v.to_string();
    }
    if let Some(v) = payload.get("chip_gap").and_then(|x| x.as_u64()) {
        overlay.chip_gap = v as u32;
    }
    if let Some(v) = payload.get("chip_pad_v").and_then(|x| x.as_u64()) {
        overlay.chip_pad_v = v as u32;
    }
    if let Some(v) = payload.get("chip_pad_h").and_then(|x| x.as_u64()) {
        overlay.chip_pad_h = v as u32;
    }
    if let Some(v) = payload.get("chip_radius").and_then(|x| x.as_u64()) {
        overlay.chip_radius = v as u32;
    }
    if let Some(v) = payload.get("chip_font_px").and_then(|x| x.as_u64()) {
        overlay.chip_font_px = v as u32;
    }
    if let Some(v) = payload.get("chip_font_weight").and_then(|x| x.as_u64()) {
        overlay.chip_font_weight = v as u32;
    }
    if let Some(v) = payload.get("background").and_then(|x| x.as_str()) {
        overlay.background = v.to_string();
    }
    if let Some(v) = payload.get("cols").and_then(|x| x.as_u64()) {
        overlay.cols = v as u32;
    }
    if let Some(v) = payload.get("rows").and_then(|x| x.as_u64()) {
        overlay.rows = v as u32;
    }
    if let Some(v) = payload.get("align").and_then(|x| x.as_str()) {
        overlay.align = v.to_string();
    }
    if let Some(v) = payload.get("direction").and_then(|x| x.as_str()) {
        overlay.direction = v.to_string();
    }

    // Save overlay config to registry
    let _ = crate::settings::save_overlay_config(
        overlay.fade_in_ms,
        overlay.fade_out_ms,
        &overlay.chip_bg,
        &overlay.chip_fg,
        overlay.chip_gap,
        overlay.chip_pad_v,
        overlay.chip_pad_h,
        overlay.chip_radius,
        overlay.chip_font_px,
        overlay.chip_font_weight,
        &overlay.background,
        overlay.cols,
        overlay.rows,
        &overlay.align,
        &overlay.direction,
    );

    Json(json!({ "ok": true }))
}

async fn api_get_launcher_language(AxumState(state): AxumState<SharedState>) -> impl IntoResponse {
    let state_lock = state.read();
    Json(json!({
        "language": state_lock.language,
    }))
}

#[derive(serde::Deserialize)]
struct FocusRequest {
    hwnd: String,
}

async fn api_focus_window(Json(payload): Json<FocusRequest>) -> impl IntoResponse {
    // Parse HWND from string
    if let Ok(hwnd_num) = payload.hwnd.parse::<usize>() {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Foundation::HWND;
            use windows::Win32::UI::WindowsAndMessaging::{SetForegroundWindow, ShowWindow, SW_RESTORE};
            
            let hwnd = HWND(hwnd_num as *mut _);
            unsafe {
                // Restore window if minimized
                let _ = ShowWindow(hwnd, SW_RESTORE);
                // Bring to foreground
                let _ = SetForegroundWindow(hwnd);
            }
        }
        Json(json!({ "ok": true }))
    } else {
        Json(json!({ "ok": false, "error": "Invalid HWND" }))
    }
}

