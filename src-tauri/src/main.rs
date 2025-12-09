// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod keyboard;
mod server;
mod settings;
mod state;
mod window_info;

// macOS accessibility trust check to avoid crash when global key hook is denied
#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::process::Command; // For macOS AppleScript prompt
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, State, Wry};

use crate::server::ServerController;
use crate::settings::LauncherSettings;
use crate::state::AppState;

// Check if running as portable
fn is_portable() -> bool {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_name) = exe_path.file_name() {
            return exe_name.to_string_lossy().to_lowercase().contains("portable");
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

// Tauri Commands
#[tauri::command]
fn get_launcher_settings() -> LauncherSettings {
    LauncherSettings::load()
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

#[tauri::command]
fn minimize_to_tray(window: tauri::Window<Wry>) -> Result<(), String> {
    let app = window.app_handle();
    
    // Create tray icon if it doesn't exist
    if app.tray_by_id("main-tray").is_none() {
        let show_item = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)
            .map_err(|e| format!("Failed to create menu item: {}", e))?;
        let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)
            .map_err(|e| format!("Failed to create menu item: {}", e))?;
        let menu = Menu::with_items(app, &[&show_item, &quit_item])
            .map_err(|e| format!("Failed to create menu: {}", e))?;

        TrayIconBuilder::with_id("main-tray")
            .icon(app.default_window_icon().unwrap().clone())
            .menu(&menu)
            .show_menu_on_left_click(false)
            .on_menu_event(|app, event| {
                match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                            let _ = window.set_skip_taskbar(false);
                        }
                        // Remove tray icon
                        let _ = app.remove_tray_by_id("main-tray");
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                }
            })
            .on_tray_icon_event(|tray, event| {
                match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } | TrayIconEvent::DoubleClick {
                        button: MouseButton::Left,
                        ..
                    } => {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                            let _ = window.set_skip_taskbar(false);
                        }
                        // Remove tray icon
                        let _ = app.remove_tray_by_id("main-tray");
                    }
                    _ => {}
                }
            })
            .build(app)
            .map_err(|e| format!("Failed to build tray: {}", e))?;
    }
    
    // Hide window and remove from taskbar
    window.hide().map_err(|e| format!("Failed to hide window: {}", e))?;
    window.set_skip_taskbar(true).map_err(|e| format!("Failed to set skip taskbar: {}", e))?;
    
    Ok(())
}

#[tauri::command]
fn set_run_on_startup(enabled: bool) -> Result<(), String> {
    let mut settings = LauncherSettings::load();
    settings.run_on_startup = enabled;
    settings.save()?;
    settings::set_windows_startup(enabled)
}

#[tauri::command]
fn reset_settings() -> Result<(), String> {
    settings::reset_all_settings()
}

fn main() {
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
            eprintln!("Warning: Failed to create single instance lock (continuing anyway): {}", e);
            println!("Note: Multiple instances may be able to run simultaneously");
        }
    }

    // Create application state with settings from registry
    let mut initial_state = AppState::new();
    let settings = LauncherSettings::load();
    initial_state.language = settings.language.clone();
    println!("Initial language: {}", initial_state.language);
    
    // Load target config from registry
    let (target_mode, target_value) = settings::load_target_config();
    initial_state.target_config.mode = target_mode;
    initial_state.target_config.value = target_value;
    println!("Loaded target config: mode={}, value={:?}", initial_state.target_config.mode, initial_state.target_config.value);
    
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
    let (fade_in_ms, fade_out_ms, chip_bg, chip_fg, chip_gap, chip_pad_v, chip_pad_h,
         chip_radius, chip_font_px, chip_font_weight, background, cols, rows, align, direction) 
        = settings::load_overlay_config();
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
    println!("Loaded overlay config from registry");
    
    let app_state = Arc::new(RwLock::new(initial_state));

    // Create server controller
    let server_controller = Arc::new(Mutex::new(ServerController::new()));

    // Start keyboard hook in background (with permission check on macOS)
    #[cfg(target_os = "macos")]
    {
        let disable_hook_env = std::env::var("KV_DISABLE_HOOK").unwrap_or_default();
        let disable_hook = disable_hook_env == "1" || disable_hook_env.eq_ignore_ascii_case("true");

        if disable_hook {
            eprintln!("KV_DISABLE_HOOK=1 set. Keyboard hook disabled on macOS.");
        } else {
            // Check Accessibility permission (this is the main one we can verify programmatically)
            let accessibility_granted = unsafe { AXIsProcessTrusted() };
            eprintln!("[macOS Permissions] Accessibility: {}", if accessibility_granted { "✓ Granted" } else { "✗ Missing" });
            
            if accessibility_granted {
                eprintln!("Accessibility permission granted. Starting keyboard hook on macOS.");
                eprintln!("NOTE: If window titles are empty, enable Screen Recording permission!");
                eprintln!("System Settings > Privacy & Security > Screen Recording > KeyQueueViewer");
                
                let state_clone = app_state.clone();
                std::thread::spawn(move || {
                    keyboard::start_keyboard_hook(state_clone);
                });
            } else {
                // Accessibility permission missing - show unified permission setup dialog
                eprintln!("Accessibility permission missing - showing permission setup guide...");
                
                // Show single dialog with buttons for each permission
                loop {
                    // Check current permission status
                    let acc_status = if unsafe { AXIsProcessTrusted() } { "✓" } else { "✗" };
                    
                    let dialog_text = format!(
                        r#"display dialog "🔐 KeyQueueViewer Permission Setup

Required permissions:
{} Accessibility (keyboard capture)
✗ Input Monitoring (key events)  
✗ Screen Recording (window titles)

Click a button to open that setting.
After enabling ALL permissions, click 'Done & Restart'.

⚠️ App will restart after clicking 'Done & Restart'!" with title "Permission Setup" buttons {{"1. Accessibility", "2. Input Monitor", "3. Screen Record"}} default button "1. Accessibility""#,
                        acc_status
                    );
                    
                    let result = Command::new("osascript").args(["-e", &dialog_text]).output();
                    
                    if let Ok(output) = result {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        
                        if stdout.contains("1. Accessibility") {
                            let _ = open::that("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility");
                        } else if stdout.contains("2. Input Monitor") {
                            let _ = open::that("x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent");
                        } else if stdout.contains("3. Screen Record") {
                            let _ = open::that("x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture");
                        } else {
                            // Dialog was cancelled
                            break;
                        }
                        
                        // Wait a bit then show "Done" dialog
                        std::thread::sleep(std::time::Duration::from_millis(500));
                        
                        let done_result = Command::new("osascript").args([
                            "-e",
                            r#"display dialog "Did you enable the permission?

Click 'More Settings' to configure another permission.
Click 'Done & Restart' when ALL permissions are enabled." with title "Permission Setup" buttons {"More Settings", "Done & Restart"} default button "More Settings""#,
                        ]).output();
                        
                        if let Ok(done_output) = done_result {
                            let done_stdout = String::from_utf8_lossy(&done_output.stdout);
                            if done_stdout.contains("Done & Restart") {
                                // Show final message and exit
                                let _ = Command::new("osascript").args([
                                    "-e",
                                    r#"display dialog "Restarting KeyQueueViewer...

The app will now restart to apply permissions." with title "Restarting" buttons {"OK"} default button "OK" giving up after 2"#,
                                ]).output();
                                
                                eprintln!("User requested restart after permission setup.");
                                break;
                            }
                            // Otherwise continue the loop to show permission dialog again
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                
                eprintln!("Permission setup dialog closed. Please restart the app if permissions were changed.");
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
        .setup(|app| {
            // Setup window event handlers
            if let Some(window) = app.get_webview_window("main") {
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { .. } = event {
                        // Just close the app - no tray minimization
                        // User can use "Minimize to Tray" button if they want
                        std::process::exit(0);
                    }
                });
                
                /* OLD CODE - tray on close
                let app_handle = app.app_handle().clone();
                let window_clone = window.clone();
                
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        if app_handle.tray_by_id("main-tray").is_none() {
                            let show_item = MenuItem::with_id(&app_handle, "show", "Show Window", true, None::<&str>).unwrap();
                            let quit_item = MenuItem::with_id(&app_handle, "quit", "Quit", true, None::<&str>).unwrap();
                            let menu = Menu::with_items(&app_handle, &[&show_item, &quit_item]).unwrap();

                            let tray = TrayIconBuilder::with_id("main-tray")
                                .icon(app_handle.default_window_icon().unwrap().clone())
                                .menu(&menu)
                                .menu_on_left_click(false)
                                .on_menu_event(|app, event| {
                                    match event.id.as_ref() {
                                        "show" => {
                                            if let Some(window) = app.get_webview_window("main") {
                                                let _ = window.show();
                                                let _ = window.set_focus();
                                                let _ = window.set_skip_taskbar(false);
                                            }
                                            // Remove tray icon when window is shown
                                            if let Some(tray) = app.tray_by_id("main-tray") {
                                                let _ = app.remove_tray_by_id("main-tray");
                                            }
                                        }
                                        "quit" => {
                                            app.exit(0);
                                        }
                                        _ => {}
                                    }
                                })
                                .on_tray_icon_event(|tray, event| {
                                    match event {
                                        TrayIconEvent::Click {
                                            button: MouseButton::Left,
                                            button_state: MouseButtonState::Up,
                                            ..
                                        } | TrayIconEvent::DoubleClick {
                                            button: MouseButton::Left,
                                            ..
                                        } => {
                                            let app = tray.app_handle();
                                            if let Some(window) = app.get_webview_window("main") {
                                                let _ = window.show();
                                                let _ = window.set_focus();
                                                let _ = window.set_skip_taskbar(false);
                                            }
                                            // Remove tray icon when window is shown
                                            let _ = app.remove_tray_by_id("main-tray");
                                        }
                                        _ => {}
                                    }
                                })
                                .build(&app_handle);
                            
                            std::mem::forget(tray); // Keep tray alive
                        }
                        
                        let _ = window_clone.hide();
                        let _ = window_clone.set_skip_taskbar(true);
                        api.prevent_close();
                    }
                });
                */
            }
            Ok(())
        })
        .manage(app_handle)
        .invoke_handler(tauri::generate_handler![
            get_launcher_settings,
            save_port_setting,
            save_language_setting,
            get_server_status,
            start_server,
            stop_server,
            open_url,
            minimize_to_tray,
            set_run_on_startup,
            reset_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

