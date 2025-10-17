use crate::state::{AppState, TargetConfig};
use crate::window_info;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State as AxumState, WebSocketUpgrade,
    },
    http::{header, StatusCode},
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use parking_lot::RwLock;
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

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
            return Err("Server is already running".to_string());
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
            return Err("Server is not running".to_string());
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
        .route("/control", get(get_control))
        .route("/ws", get(websocket_handler))
        .route("/api/windows", get(api_windows))
        .route("/api/foreground", get(api_foreground))
        .route("/api/target", get(api_get_target))
        .route("/api/target", post(api_set_target))
        .route("/api/config", get(api_get_config))
        .route("/api/config", post(api_set_config))
        .route("/api/overlay-config", get(api_get_overlay_config))
        .route("/api/overlay-config", post(api_set_overlay_config))
        .route("/api/launcher-language", get(api_get_launcher_language))
        .route("/api/focus", post(api_focus_window))
        .nest_service("/static", ServeDir::new("ui"))
        .layer(cors)
        .with_state(state)
}

async fn root_redirect() -> impl IntoResponse {
    (StatusCode::FOUND, [(header::LOCATION, "/control")])
}

async fn get_overlay() -> impl IntoResponse {
    // Try multiple possible paths for the overlay HTML file
    let possible_paths = vec![
        "../ui/overlay.html",
        "ui/overlay.html",
        "../../ui/overlay.html",
        "../../../ui/overlay.html",
    ];
    
    for path in possible_paths {
        if let Ok(content) = tokio::fs::read_to_string(path).await {
            return Html(content).into_response();
        }
    }
    
    (StatusCode::NOT_FOUND, "Overlay not found").into_response()
}

async fn get_control() -> impl IntoResponse {
    // Try multiple possible paths for the UI files
    let possible_paths = vec![
        "../ui/control.html",
        "ui/control.html",
        "../../ui/control.html",
        "../../../ui/control.html",
    ];
    
    for path in possible_paths {
        if let Ok(content) = tokio::fs::read_to_string(path).await {
            return Html(content).into_response();
        }
    }
    
    (StatusCode::NOT_FOUND, "Control not found").into_response()
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

