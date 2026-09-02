// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_update;
mod console_log;
mod keyboard;
#[cfg(target_os = "linux")]
mod linux_desktop;
#[cfg(target_os = "linux")]
mod linux_input;
mod server;
mod settings;
mod state;
mod tray;
mod window_info;

// macOS accessibility trust check to avoid crash when global key hook is denied
#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tauri::{Manager, State, Wry};

use crate::server::ServerController;
use crate::settings::LauncherSettings;
use crate::state::AppState;

#[cfg(target_os = "windows")]
use std::collections::HashSet;
#[cfg(target_os = "windows")]
use std::ffi::OsStr;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[cfg(target_os = "windows")]
fn hidden_command(program: &str) -> Command {
    let mut command = Command::new(program);
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

#[cfg(target_os = "windows")]
fn is_running_as_admin() -> bool {
    use windows::Win32::Security::{
        GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
    };
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    unsafe {
        let mut token = Default::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION::default();
        let mut returned_len = 0u32;
        if GetTokenInformation(
            token,
            TokenElevation,
            Some((&mut elevation as *mut TOKEN_ELEVATION).cast()),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut returned_len,
        )
        .is_err()
        {
            return false;
        }

        elevation.TokenIsElevated != 0
    }
}

#[cfg(target_os = "windows")]
fn wide_null(value: &OsStr) -> Vec<u16> {
    value.encode_wide().chain(std::iter::once(0)).collect()
}

#[cfg(target_os = "windows")]
fn quote_windows_arg(arg: &OsStr) -> String {
    let value = arg.to_string_lossy();
    if value.is_empty() || value.chars().any(|c| c.is_whitespace() || c == '"') {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.into_owned()
    }
}

#[cfg(target_os = "windows")]
fn try_relaunch_as_admin() -> Result<(), String> {
    use windows::core::{w, PCWSTR};
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get current executable path: {}", e))?;
    let working_dir = exe_path
        .parent()
        .ok_or_else(|| "Failed to get executable directory".to_string())?;
    let args_joined = std::env::args_os()
        .skip(1)
        .map(|arg| quote_windows_arg(&arg))
        .collect::<Vec<_>>()
        .join(" ");

    let exe_path_w = wide_null(exe_path.as_os_str());
    let working_dir_w = wide_null(working_dir.as_os_str());
    let args_w = wide_null(OsStr::new(&args_joined));
    let args_ptr = if args_joined.is_empty() {
        PCWSTR::null()
    } else {
        PCWSTR(args_w.as_ptr())
    };

    let result = unsafe {
        ShellExecuteW(
            None,
            w!("runas"),
            PCWSTR(exe_path_w.as_ptr()),
            args_ptr,
            PCWSTR(working_dir_w.as_ptr()),
            SW_SHOWNORMAL,
        )
    };
    let code = result.0 as usize;

    if code > 32 {
        Ok(())
    } else {
        Err(format!(
            "Elevation prompt was canceled or failed (ShellExecuteW error {}).",
            code
        ))
    }
}

// Check if running as portable
fn is_portable() -> bool {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_name) = exe_path.file_name() {
            return exe_name
                .to_string_lossy()
                .to_lowercase()
                .contains("portable");
        }
    }
    false
}

// Application state that holds server controller and app state
struct AppHandle {
    server_controller: Arc<Mutex<ServerController>>,
    app_state: Arc<RwLock<AppState>>,
}

// Response types for commands
#[derive(Debug, Serialize, Deserialize)]
struct ServerStatus {
    running: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortProcessInfo {
    pid: u32,
    name: String,
}

#[derive(Debug, Serialize)]
struct AppUpdateCheckResult {
    current_version: String,
    update: Option<app_update::ReleaseUpdate>,
}

// macOS permission status
#[derive(Debug, Serialize, Deserialize, Clone)]
struct MacOSPermissions {
    accessibility: bool,
    input_monitoring: bool,
    screen_recording: bool,
}

// macOS Screen Recording permission check
#[cfg(target_os = "macos")]
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGPreflightScreenCaptureAccess() -> bool;
}

// Tauri Commands
#[tauri::command]
fn check_macos_permissions() -> MacOSPermissions {
    #[cfg(target_os = "macos")]
    {
        let accessibility = unsafe { AXIsProcessTrusted() };

        // Check Input Monitoring by trying to create event tap
        let input_monitoring = {
            use core_graphics::event::{
                CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType,
            };
            CGEventTap::new(
                CGEventTapLocation::Session,
                CGEventTapPlacement::HeadInsertEventTap,
                CGEventTapOptions::ListenOnly,
                vec![CGEventType::KeyDown],
                |_, _, _| None,
            )
            .is_ok()
        };

        // Check Screen Recording permission
        let screen_recording = unsafe { CGPreflightScreenCaptureAccess() };

        MacOSPermissions {
            accessibility,
            input_monitoring,
            screen_recording,
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        MacOSPermissions {
            accessibility: true,
            input_monitoring: true,
            screen_recording: true,
        }
    }
}

#[tauri::command]
fn open_macos_permission_settings(permission_type: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        // Trigger API call to register app in System Settings
        match permission_type.as_str() {
            "accessibility" => {
                // AXIsProcessTrusted triggers registration
                unsafe {
                    AXIsProcessTrusted();
                }
            }
            "input_monitoring" => {
                // CGEventTap triggers Input Monitoring registration
                use core_graphics::event::{
                    CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
                    CGEventType,
                };
                let _ = CGEventTap::new(
                    CGEventTapLocation::Session,
                    CGEventTapPlacement::HeadInsertEventTap,
                    CGEventTapOptions::ListenOnly,
                    vec![CGEventType::KeyDown],
                    |_, _, _| None,
                );
            }
            "screen_recording" => {
                // CGWindowListCopyWindowInfo triggers Screen Recording registration
                use core_graphics::window::{kCGNullWindowID, kCGWindowListOptionOnScreenOnly};
                unsafe {
                    core_graphics::window::CGWindowListCopyWindowInfo(
                        kCGWindowListOptionOnScreenOnly,
                        kCGNullWindowID,
                    );
                }
            }
            _ => return Err("Unknown permission type".to_string()),
        };

        // Open System Settings
        let url = match permission_type.as_str() {
            "accessibility" => {
                "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"
            }
            "input_monitoring" => {
                "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent"
            }
            "screen_recording" => {
                "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture"
            }
            _ => return Err("Unknown permission type".to_string()),
        };
        open::that(url).map_err(|e| e.to_string())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = permission_type;
        Ok(())
    }
}

#[tauri::command]
fn get_launcher_settings() -> LauncherSettings {
    LauncherSettings::load()
}

#[tauri::command]
fn check_app_update() -> Result<AppUpdateCheckResult, String> {
    Ok(AppUpdateCheckResult {
        current_version: app_update::CURRENT_VERSION.to_string(),
        update: app_update::check_latest_release()?,
    })
}

#[tauri::command]
fn get_app_version() -> String {
    app_update::CURRENT_VERSION.to_string()
}

/// Every installable release for this platform, so the launcher can offer a
/// picker instead of only the newest build.
#[tauri::command]
fn list_app_releases() -> Result<Vec<app_update::ReleaseOption>, String> {
    app_update::list_available_releases()
}

/// Installs a chosen release, including the one already running (re-download) or
/// an older one (downgrade).
#[tauri::command]
fn install_app_version(tag: String) -> Result<(), String> {
    app_update::install_release(&tag)
}

#[tauri::command]
fn install_app_update() -> Result<(), String> {
    app_update::install_latest_update()
}

#[tauri::command]
fn save_port_setting(port: u16) -> Result<(), String> {
    let mut settings = LauncherSettings::load();
    settings.port = port;
    settings.save()
}

#[tauri::command]
fn save_language_setting(language: String, handle: State<AppHandle>) -> Result<(), String> {
    // Save to settings file
    let mut settings = LauncherSettings::load();
    settings.language = language.clone();
    settings.save()?;

    // Update AppState language
    let mut state = handle.app_state.write();
    state.language = language;

    Ok(())
}

#[tauri::command]
fn get_server_status(handle: State<AppHandle>) -> ServerStatus {
    ServerStatus {
        running: handle.server_controller.lock().is_running(),
    }
}

#[tauri::command]
fn start_server(port: u16, handle: State<AppHandle>) -> Result<(), String> {
    let mut controller = handle.server_controller.lock();
    controller.start(handle.app_state.clone(), port)
}

#[tauri::command]
fn stop_server(handle: State<AppHandle>) -> Result<(), String> {
    let mut controller = handle.server_controller.lock();
    controller.stop()
}

#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| format!("Failed to open URL: {}", e))
}

#[cfg(target_os = "windows")]
fn list_port_processes_windows(port: u16) -> Result<Vec<PortProcessInfo>, String> {
    let output = hidden_command("netstat")
        .args(["-ano", "-p", "tcp"])
        .output()
        .map_err(|e| format!("Failed to run netstat: {}", e))?;
    if !output.status.success() {
        return Err("Failed to inspect TCP ports".to_string());
    }

    let text = String::from_utf8_lossy(&output.stdout);

    let mut pids = HashSet::new();
    let suffix = format!(":{}", port);

    for raw_line in text.lines() {
        let line = raw_line.trim();
        // Skip header lines (support both English and Korean Windows)
        if !line.starts_with("TCP") {
            continue;
        }

        // Split by whitespace and collect columns
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 5 {
            continue;
        }

        // Parse columns: TCP, Local Address, Foreign Address, State, PID
        let local_addr = cols[1];
        let state = cols[3];
        let pid_str = cols[4];

        // Check if this is the port we're looking for (LISTENING state)
        if !local_addr.ends_with(&suffix) || state != "LISTENING" {
            continue;
        }

        if let Ok(pid) = pid_str.parse::<u32>() {
            eprintln!("[DEBUG] Found PID {} using port {}", pid, port);
            pids.insert(pid);
        }
    }

    let mut result = Vec::new();
    for pid in pids {
        let name = process_name_windows(pid).unwrap_or_else(|| "Unknown".to_string());
        eprintln!("[DEBUG] Process: PID={}, Name={}", pid, name);
        result.push(PortProcessInfo { pid, name });
    }
    result.sort_by_key(|p| p.pid);
    eprintln!("[DEBUG] Total processes found: {}", result.len());
    Ok(result)
}

#[cfg(target_os = "windows")]
fn process_name_windows(pid: u32) -> Option<String> {
    let filter = format!("PID eq {}", pid);
    let output = hidden_command("tasklist")
        .args(["/FI", &filter, "/FO", "CSV", "/NH"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let line = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()?
        .trim()
        .to_string();

    // Check for empty or error messages (support both English and Korean Windows)
    if line.is_empty()
        || line.contains("No tasks are running")
        || line.contains("정보:")
        || line.contains("작업이 실행되고 있지 않습니다")
        || line.starts_with("ERROR:")
    {
        return None;
    }

    // Parse CSV format: "imagename","pid","sessionname","session#","memusage"
    // The line looks like: "chrome.exe","1234","Console","1","100,000 K"
    // We need to extract the first field (image name)

    // Remove leading/trailing quotes if present
    let line = line.trim();

    // Split by CSV delimiter "," (quote-comma-quote pattern)
    // First, try to parse as proper CSV
    if let Some(stripped) = line.strip_prefix('"') {
        // Find the closing quote for the first field
        if let Some(end_quote) = stripped.find('"') {
            let name = &stripped[..end_quote];
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }

    // Fallback: split by "," and take first field
    let first = line.split(',').next()?.trim().trim_matches('"');
    if first.is_empty() {
        None
    } else {
        Some(first.to_string())
    }
}

// macOS always ships lsof; most Linux distros ship ss but not lsof, so try both.
// Without root these only report processes owned by the current user.
#[cfg(not(target_os = "windows"))]
fn lsof_listeners(port: u16) -> Vec<PortProcessInfo> {
    let Ok(output) = Command::new("lsof")
        .args(["-nP", &format!("-iTCP:{}", port), "-sTCP:LISTEN", "-Fpc"])
        .output()
    else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }

    // Field output emits `p<pid>` then `c<command>` for each process.
    let mut result = Vec::new();
    let mut pid: Option<u32> = None;
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let mut chars = line.chars();
        let Some(tag) = chars.next() else {
            continue;
        };
        let value = chars.as_str().trim();
        match tag {
            'p' => pid = value.parse().ok(),
            'c' => {
                if let Some(pid) = pid.take() {
                    result.push(PortProcessInfo {
                        pid,
                        name: value.to_string(),
                    });
                }
            }
            _ => {}
        }
    }
    result
}

#[cfg(not(target_os = "windows"))]
fn ss_listeners(port: u16) -> Vec<PortProcessInfo> {
    let Ok(output) = Command::new("ss")
        .args(["-H", "-l", "-t", "-n", "-p"])
        .output()
    else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }

    let suffix = format!(":{}", port);
    let mut result = Vec::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        // State Recv-Q Send-Q Local:Port Peer:Port users:(("name",pid=1,fd=2),...)
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 4 || !cols[3].ends_with(&suffix) {
            continue;
        }
        let Some((_, users)) = line.split_once("users:(") else {
            continue;
        };
        for chunk in users.split("),(") {
            let chunk = chunk.trim_matches(|c: char| c == '(' || c == ')');
            let mut fields = chunk.split(',');
            let name = fields.next().unwrap_or("").trim_matches('"').to_string();
            let pid = fields
                .find_map(|field| field.strip_prefix("pid="))
                .and_then(|value| value.parse::<u32>().ok());
            if let Some(pid) = pid {
                result.push(PortProcessInfo {
                    pid,
                    name: if name.is_empty() {
                        "Unknown".to_string()
                    } else {
                        name
                    },
                });
            }
        }
    }
    result
}

#[cfg(not(target_os = "windows"))]
fn list_port_processes_unix(port: u16) -> Result<Vec<PortProcessInfo>, String> {
    let mut result = lsof_listeners(port);
    if result.is_empty() {
        result = ss_listeners(port);
    }
    result.sort_by_key(|process| process.pid);
    result.dedup_by_key(|process| process.pid);
    Ok(result)
}

#[tauri::command]
fn get_port_processes(port: u16) -> Result<Vec<PortProcessInfo>, String> {
    #[cfg(target_os = "windows")]
    {
        list_port_processes_windows(port)
    }
    #[cfg(not(target_os = "windows"))]
    {
        list_port_processes_unix(port)
    }
}

/// How key capture is actually running, so the launcher can warn when the OS is
/// refusing to hand over global input instead of silently showing nothing.
#[derive(Debug, Serialize)]
struct InputStatus {
    backend: String,
    ok: bool,
    detail: String,
    devices: usize,
}

#[tauri::command]
fn get_input_status() -> InputStatus {
    #[cfg(target_os = "linux")]
    {
        let status = linux_input::status();
        let mut detail = status.detail;

        // Target filtering needs a compositor route of its own on Wayland, and
        // failing it silently looks identical to "the filter matches nothing".
        if linux_desktop::is_wayland() && linux_desktop::foreground_window_cached().is_none() {
            if !detail.is_empty() {
                detail.push(' ');
            }
            detail.push_str(
                "Window targeting cannot see Wayland windows here. GNOME needs the                  window-calls-extended shell extension; KDE needs kdotool. Hyprland and                  sway work out of the box.",
            );
        }

        InputStatus {
            backend: status.backend,
            ok: status.ok && detail.is_empty(),
            detail,
            devices: status.devices,
        }
    }

    #[cfg(target_os = "windows")]
    {
        InputStatus {
            backend: "polling".to_string(),
            ok: true,
            detail: String::new(),
            devices: 0,
        }
    }

    #[cfg(target_os = "macos")]
    {
        let permissions = check_macos_permissions();
        let ok = permissions.accessibility && permissions.input_monitoring;
        InputStatus {
            backend: "cgeventtap".to_string(),
            ok,
            detail: if ok {
                String::new()
            } else {
                "Grant Accessibility and Input Monitoring in System Settings > Privacy & Security."
                    .to_string()
            },
            devices: 0,
        }
    }
}

/// Reserve a port from Windows dynamic port allocation
/// This prevents Windows from giving this port to other services
#[cfg(target_os = "windows")]
fn reserve_port_from_windows(port: u16) -> Result<(), String> {
    // First, try to get current excluded port ranges
    let output = hidden_command("netsh")
        .args(["int", "ipv4", "show", "excludedportrange", "protocol=tcp"])
        .output()
        .map_err(|e| format!("Failed to check excluded ports: {}", e))?;

    let text = String::from_utf8_lossy(&output.stdout);

    // Check if our port is already in excluded range
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("Start") || line.starts_with("-") {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            if let (Ok(start), Ok(end)) = (parts[0].parse::<u16>(), parts[1].parse::<u16>()) {
                if port >= start && port <= end {
                    // Port is in Windows excluded range - try to remove it
                    eprintln!("[Port] Port {} is in Windows excluded range {}-{}, attempting to reserve...", port, start, end);

                    // Stop WinNAT service temporarily to modify excluded ports
                    let _ = hidden_command("net").args(["stop", "winnat"]).output();

                    // Start WinNAT again
                    let _ = hidden_command("net").args(["start", "winnat"]).output();

                    eprintln!("[Port] Restarted WinNAT service to clear excluded port ranges");
                    break;
                }
            }
        }
    }

    Ok(())
}

/// Static version for use in other modules
#[cfg(target_os = "windows")]
pub fn get_port_processes_static(port: u16) -> Vec<PortProcessInfo> {
    list_port_processes_windows(port).unwrap_or_default()
}

#[cfg(not(target_os = "windows"))]
pub fn get_port_processes_static(port: u16) -> Vec<PortProcessInfo> {
    let _ = port;
    Vec::new()
}

#[tauri::command]
fn kill_process(pid: u32) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let status = hidden_command("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .status()
            .map_err(|e| format!("Failed to run taskkill: {}", e))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("Failed to terminate PID {}", pid))
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let pid_arg = pid.to_string();
        let _ = Command::new("kill").args(["-TERM", &pid_arg]).status();

        // Escalate only if the process ignored the polite signal, so freeing a port
        // ends up as reliable as taskkill /F on Windows.
        for _ in 0..10 {
            std::thread::sleep(Duration::from_millis(100));
            let alive = Command::new("kill")
                .args(["-0", &pid_arg])
                .status()
                .map(|status| status.success())
                .unwrap_or(false);
            if !alive {
                return Ok(());
            }
        }

        let status = Command::new("kill")
            .args(["-KILL", &pid_arg])
            .status()
            .map_err(|e| format!("Failed to run kill: {}", e))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("Failed to terminate PID {}", pid))
        }
    }
}

fn try_stop_server(app: &tauri::AppHandle<Wry>) {
    if let Some(state) = app.try_state::<AppHandle>() {
        let mut controller = state.server_controller.lock();
        if controller.is_running() {
            let _ = controller.stop();
        }
    }
}

fn open_service_url(path: &str) {
    static LAST_OPEN: OnceLock<Mutex<Option<(String, Instant)>>> = OnceLock::new();
    let gate = LAST_OPEN.get_or_init(|| Mutex::new(None));
    {
        let mut last = gate.lock();
        if let Some((last_path, when)) = &*last {
            if last_path == path && when.elapsed() < Duration::from_millis(900) {
                return;
            }
        }
        *last = Some((path.to_string(), Instant::now()));
    }

    let settings = LauncherSettings::load();
    let url = format!("http://localhost:{}{}", settings.port, path);
    let _ = open::that(url);
}

#[tauri::command]
fn minimize_to_tray(window: tauri::Window<Wry>) -> Result<(), String> {
    // The tray icon lives for the whole session (see tray::init), so hiding the
    // window is all this has to do.
    window
        .hide()
        .map_err(|e| format!("Failed to hide window: {}", e))?;
    window
        .set_skip_taskbar(true)
        .map_err(|e| format!("Failed to set skip taskbar: {}", e))?;

    Ok(())
}

#[tauri::command]
fn set_run_on_startup(enabled: bool) -> Result<(), String> {
    let mut settings = LauncherSettings::load();
    let previous_enabled = settings.run_on_startup;
    settings.run_on_startup = enabled;

    settings::set_startup_registration(enabled)?;
    if let Err(err) = settings.save() {
        let _ = settings::set_startup_registration(previous_enabled);
        return Err(err);
    }

    Ok(())
}

/// Feeds the launcher's log panel. `since` is the cursor from the previous call.
#[tauri::command]
fn get_console_log(since: u64) -> console_log::LogChunk {
    console_log::read_since(since)
}

#[tauri::command]
fn reset_settings() -> Result<(), String> {
    settings::reset_all_settings()
}

fn main() {
    // Before anything prints, so the launcher's log panel sees startup too.
    console_log::capture();

    #[cfg(target_os = "windows")]
    {
        let disable_auto_elevate = std::env::var("KV_NO_AUTO_ELEVATE").unwrap_or_default();
        let disable_auto_elevate =
            disable_auto_elevate == "1" || disable_auto_elevate.eq_ignore_ascii_case("true");

        if !disable_auto_elevate && !is_running_as_admin() {
            match try_relaunch_as_admin() {
                Ok(()) => {
                    std::process::exit(0);
                }
                Err(err) => {
                    eprintln!(
                        "Warning: Failed to relaunch as administrator (continuing without elevation): {}",
                        err
                    );
                }
            }
        }
    }

    // Prevent multiple instances (with safe error handling for macOS)
    let app_name = if is_portable() {
        "keyviewer-portable"
    } else {
        "keyviewer"
    };

    // Try to create single instance lock, but don't panic if it fails (e.g. on macOS with permission issues)
    match single_instance::SingleInstance::new(app_name) {
        Ok(instance) => {
            if !instance.is_single() {
                eprintln!("Another instance is already running!");
                std::process::exit(1);
            }
            // Keep instance alive
            std::mem::forget(instance);
            println!("Single instance lock acquired successfully");
        }
        Err(e) => {
            // Log the error but continue - single instance check is not critical for functionality
            eprintln!(
                "Warning: Failed to create single instance lock (continuing anyway): {}",
                e
            );
            println!("Note: Multiple instances may be able to run simultaneously");
        }
    }

    // Create application state with settings from registry
    let mut initial_state = AppState::new();
    let settings = LauncherSettings::load();
    initial_state.language = settings.language.clone();
    initial_state.app_config.port = settings.port;
    println!("Initial language: {}", initial_state.language);

    #[cfg(not(debug_assertions))]
    {
        if settings.run_on_startup {
            if let Err(e) = settings::set_startup_registration(true) {
                eprintln!("[Startup] Failed to repair startup registration: {}", e);
            }
        }
    }

    // Try to reserve the port from Windows dynamic allocation
    #[cfg(target_os = "windows")]
    {
        let port = settings.port;
        eprintln!("[Port] Checking if port {} is available...", port);
        if let Err(e) = reserve_port_from_windows(port) {
            eprintln!("[Port] Warning: Could not check/reserve port: {}", e);
        }
    }

    // Load target config from registry
    let (target_mode, target_value) = settings::load_target_config();
    initial_state.target_config.mode = target_mode;
    initial_state.target_config.value = target_value;
    println!(
        "Loaded target config: mode={}, value={:?}",
        initial_state.target_config.mode, initial_state.target_config.value
    );

    // Important debug info for macOS
    #[cfg(target_os = "macos")]
    {
        println!("\n=== macOS Debug Info ===");
        println!("Target Mode: {}", initial_state.target_config.mode);
        println!("Target Value: {:?}", initial_state.target_config.value);
        println!("NOTE: If you change target settings, you may need to restart the app!");
        println!("======================\n");
    }

    // Load overlay config from registry
    let (
        fade_in_ms,
        fade_out_ms,
        chip_bg,
        chip_fg,
        chip_gap,
        chip_pad_v,
        chip_pad_h,
        chip_radius,
        chip_font_px,
        chip_font_weight,
        background,
        cols,
        rows,
        align,
        direction,
        color_mode,
        grad_color1,
        grad_color2,
        grad_dir,
        overlay_mode,
    ) = settings::load_overlay_config();
    initial_state.app_config.overlay.fade_in_ms = fade_in_ms;
    initial_state.app_config.overlay.fade_out_ms = fade_out_ms;
    initial_state.app_config.overlay.chip_bg = chip_bg;
    initial_state.app_config.overlay.chip_fg = chip_fg;
    initial_state.app_config.overlay.chip_gap = chip_gap;
    initial_state.app_config.overlay.chip_pad_v = chip_pad_v;
    initial_state.app_config.overlay.chip_pad_h = chip_pad_h;
    initial_state.app_config.overlay.chip_radius = chip_radius;
    initial_state.app_config.overlay.chip_font_px = chip_font_px;
    initial_state.app_config.overlay.chip_font_weight = chip_font_weight;
    initial_state.app_config.overlay.background = background;
    initial_state.app_config.overlay.cols = cols;
    initial_state.app_config.overlay.rows = rows;
    initial_state.app_config.overlay.align = align;
    initial_state.app_config.overlay.direction = direction;
    initial_state.app_config.overlay.color_mode = color_mode;
    initial_state.app_config.overlay.grad_color1 = grad_color1;
    initial_state.app_config.overlay.grad_color2 = grad_color2;
    initial_state.app_config.overlay.grad_dir = grad_dir;
    initial_state.app_config.overlay.overlay_mode = overlay_mode;
    println!("Loaded overlay config from registry");

    // Load key images config from file
    initial_state.app_config.key_images = settings::load_key_images_config();
    println!("Loaded key images config");

    // Load key style config from file
    initial_state.app_config.key_style = settings::load_key_style_config();
    println!("Loaded key style config");

    let app_state = Arc::new(RwLock::new(initial_state));

    // Create server controller
    let server_controller = Arc::new(Mutex::new(ServerController::new()));
    let server_running = server_controller.lock().running_flag();

    if let Err(e) = server_controller
        .lock()
        .start(app_state.clone(), settings.port)
    {
        eprintln!("[Server] Failed to auto-start on launch: {}", e);
    }

    // Start keyboard hook in background (with permission check on macOS)
    #[cfg(target_os = "macos")]
    {
        let disable_hook_env = std::env::var("KV_DISABLE_HOOK").unwrap_or_default();
        let disable_hook = disable_hook_env == "1" || disable_hook_env.eq_ignore_ascii_case("true");

        if disable_hook {
            eprintln!("KV_DISABLE_HOOK=1 set. Keyboard hook disabled on macOS.");
        } else {
            // Check all permissions
            let accessibility = unsafe { AXIsProcessTrusted() };
            let input_monitoring = {
                use core_graphics::event::{
                    CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
                    CGEventType,
                };
                CGEventTap::new(
                    CGEventTapLocation::Session,
                    CGEventTapPlacement::HeadInsertEventTap,
                    CGEventTapOptions::ListenOnly,
                    vec![CGEventType::KeyDown],
                    |_, _, _| None,
                )
                .is_ok()
            };
            let screen_recording = unsafe { CGPreflightScreenCaptureAccess() };

            eprintln!(
                "[macOS Permissions] Accessibility: {}",
                if accessibility { "✓" } else { "✗" }
            );
            eprintln!(
                "[macOS Permissions] Input Monitoring: {}",
                if input_monitoring { "✓" } else { "✗" }
            );
            eprintln!(
                "[macOS Permissions] Screen Recording: {}",
                if screen_recording { "✓" } else { "✗" }
            );

            if accessibility && input_monitoring && screen_recording {
                eprintln!("All permissions granted. Starting keyboard hook on macOS.");

                let state_clone = app_state.clone();
                std::thread::spawn(move || {
                    keyboard::start_keyboard_hook(state_clone);
                });
            } else {
                // Permission missing - Web UI will redirect to permissions.html
                eprintln!("Some permissions missing. Web UI will show permission setup page.");
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        let state_clone = app_state.clone();
        std::thread::spawn(move || {
            keyboard::start_keyboard_hook(state_clone);
        });
    }

    // Create app handle
    let app_handle = AppHandle {
        server_controller,
        app_state,
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(move |app| {
            // Tray icon carries the server state for the whole session.
            tray::init(app.handle(), server_running);

            // Setup window event handlers
            if let Some(window) = app.get_webview_window("main") {
                let app_handle = window.app_handle().clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { .. } = event {
                        try_stop_server(&app_handle);
                        app_handle.exit(0);
                    }
                });
            }
            Ok(())
        })
        .manage(app_handle)
        .invoke_handler(tauri::generate_handler![
            get_launcher_settings,
            check_app_update,
            get_app_version,
            install_app_update,
            list_app_releases,
            install_app_version,
            save_port_setting,
            save_language_setting,
            get_server_status,
            start_server,
            stop_server,
            get_port_processes,
            get_input_status,
            kill_process,
            open_url,
            minimize_to_tray,
            set_run_on_startup,
            get_console_log,
            reset_settings,
            check_macos_permissions,
            open_macos_permission_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
