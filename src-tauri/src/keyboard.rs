use crate::state::AppState;
use crate::window_info;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;

#[cfg(target_os = "linux")]
use std::sync::mpsc;

// Linux uses hook-based input
#[cfg(target_os = "linux")]
use rdev::{listen, Event, EventType, Key, Button};

// Windows uses pure polling - no rdev hook needed

#[cfg(target_os = "macos")]
use rdev::Key;

// Interval for polling actual key state (primary method for games)
// 16ms ≈ 60fps, fast enough for responsive key display
#[cfg(target_os = "windows")]
const KEY_POLLING_INTERVAL_MS: u64 = 16;

// Event types for internal processing (Linux hook-based)
#[cfg(target_os = "linux")]
#[derive(Debug, Clone)]
enum InputEvent {
    KeyPress { code: u32, label: String },
    KeyRelease { code: u32 },
    ButtonPress { code: u32, label: String },
    ButtonRelease { code: u32 },
}

// Convert rdev::Key to a display label (Linux only - Windows uses polling)
#[cfg(target_os = "linux")]
fn key_to_label(key: Key) -> String {
    match key {
        Key::Alt => "ALT".to_string(),
        Key::AltGr => "ALT GR".to_string(),
        Key::Backspace => "BKSP".to_string(),
        Key::CapsLock => "CAPS".to_string(),
        Key::ControlLeft | Key::ControlRight => "CTRL".to_string(),
        Key::Delete => "DEL".to_string(),
        Key::DownArrow => "DOWN".to_string(),
        Key::End => "END".to_string(),
        Key::Escape => "ESC".to_string(),
        Key::F1 => "F1".to_string(),
        Key::F2 => "F2".to_string(),
        Key::F3 => "F3".to_string(),
        Key::F4 => "F4".to_string(),
        Key::F5 => "F5".to_string(),
        Key::F6 => "F6".to_string(),
        Key::F7 => "F7".to_string(),
        Key::F8 => "F8".to_string(),
        Key::F9 => "F9".to_string(),
        Key::F10 => "F10".to_string(),
        Key::F11 => "F11".to_string(),
        Key::F12 => "F12".to_string(),
        Key::Home => "HOME".to_string(),
        Key::LeftArrow => "LEFT".to_string(),
        Key::MetaLeft | Key::MetaRight => "CMD".to_string(),
        Key::PageDown => "PG DN".to_string(),
        Key::PageUp => "PG UP".to_string(),
        Key::Return => "ENTER".to_string(),
        Key::RightArrow => "RIGHT".to_string(),
        Key::ShiftLeft | Key::ShiftRight => "SHIFT".to_string(),
        Key::Space => "SPACE".to_string(),
        Key::Tab => "TAB".to_string(),
        Key::UpArrow => "UP".to_string(),
        Key::PrintScreen => "PRINT".to_string(),
        Key::ScrollLock => "SCROLL".to_string(),
        Key::Pause => "PAUSE".to_string(),
        Key::NumLock => "NUM".to_string(),
        Key::BackQuote => "`".to_string(),
        Key::Num1 => "1".to_string(),
        Key::Num2 => "2".to_string(),
        Key::Num3 => "3".to_string(),
        Key::Num4 => "4".to_string(),
        Key::Num5 => "5".to_string(),
        Key::Num6 => "6".to_string(),
        Key::Num7 => "7".to_string(),
        Key::Num8 => "8".to_string(),
        Key::Num9 => "9".to_string(),
        Key::Num0 => "0".to_string(),
        Key::Minus => "-".to_string(),
        Key::Equal => "=".to_string(),
        Key::KeyQ => "Q".to_string(),
        Key::KeyW => "W".to_string(),
        Key::KeyE => "E".to_string(),
        Key::KeyR => "R".to_string(),
        Key::KeyT => "T".to_string(),
        Key::KeyY => "Y".to_string(),
        Key::KeyU => "U".to_string(),
        Key::KeyI => "I".to_string(),
        Key::KeyO => "O".to_string(),
        Key::KeyP => "P".to_string(),
        Key::LeftBracket => "[".to_string(),
        Key::RightBracket => "]".to_string(),
        Key::KeyA => "A".to_string(),
        Key::KeyS => "S".to_string(),
        Key::KeyD => "D".to_string(),
        Key::KeyF => "F".to_string(),
        Key::KeyG => "G".to_string(),
        Key::KeyH => "H".to_string(),
        Key::KeyJ => "J".to_string(),
        Key::KeyK => "K".to_string(),
        Key::KeyL => "L".to_string(),
        Key::SemiColon => ";".to_string(),
        Key::Quote => "'".to_string(),
        Key::BackSlash => "\\".to_string(),
        Key::IntlBackslash => "\\".to_string(),
        Key::KeyZ => "Z".to_string(),
        Key::KeyX => "X".to_string(),
        Key::KeyC => "C".to_string(),
        Key::KeyV => "V".to_string(),
        Key::KeyB => "B".to_string(),
        Key::KeyN => "N".to_string(),
        Key::KeyM => "M".to_string(),
        Key::Comma => ",".to_string(),
        Key::Dot => ".".to_string(),
        Key::Slash => "/".to_string(),
        Key::Insert => "INS".to_string(),
        Key::KpReturn => "ENTER".to_string(),
        Key::KpMinus => "-".to_string(),
        Key::KpPlus => "+".to_string(),
        Key::KpMultiply => "*".to_string(),
        Key::KpDivide => "/".to_string(),
        Key::Kp0 => "0".to_string(),
        Key::Kp1 => "1".to_string(),
        Key::Kp2 => "2".to_string(),
        Key::Kp3 => "3".to_string(),
        Key::Kp4 => "4".to_string(),
        Key::Kp5 => "5".to_string(),
        Key::Kp6 => "6".to_string(),
        Key::Kp7 => "7".to_string(),
        Key::Kp8 => "8".to_string(),
        Key::Kp9 => "9".to_string(),
        Key::KpDelete => "DEL".to_string(),
        _ => format!("{:?}", key).to_uppercase(),
    }
}

// macOS keycode to label mapping
#[cfg(target_os = "macos")]
fn keycode_to_label(keycode: u16) -> String {
    match keycode {
        0 => "A".to_string(),
        1 => "S".to_string(),
        2 => "D".to_string(),
        3 => "F".to_string(),
        4 => "H".to_string(),
        5 => "G".to_string(),
        6 => "Z".to_string(),
        7 => "X".to_string(),
        8 => "C".to_string(),
        9 => "V".to_string(),
        11 => "B".to_string(),
        12 => "Q".to_string(),
        13 => "W".to_string(),
        14 => "E".to_string(),
        15 => "R".to_string(),
        16 => "Y".to_string(),
        17 => "T".to_string(),
        18 => "1".to_string(),
        19 => "2".to_string(),
        20 => "3".to_string(),
        21 => "4".to_string(),
        22 => "6".to_string(),
        23 => "5".to_string(),
        24 => "=".to_string(),
        25 => "9".to_string(),
        26 => "7".to_string(),
        27 => "-".to_string(),
        28 => "8".to_string(),
        29 => "0".to_string(),
        30 => "]".to_string(),
        31 => "O".to_string(),
        32 => "U".to_string(),
        33 => "[".to_string(),
        34 => "I".to_string(),
        35 => "P".to_string(),
        36 => "ENTER".to_string(),
        37 => "L".to_string(),
        38 => "J".to_string(),
        39 => "'".to_string(),
        40 => "K".to_string(),
        41 => ";".to_string(),
        42 => "\\".to_string(),
        43 => ",".to_string(),
        44 => "/".to_string(),
        45 => "N".to_string(),
        46 => "M".to_string(),
        47 => ".".to_string(),
        48 => "TAB".to_string(),
        49 => "SPACE".to_string(),
        50 => "`".to_string(),
        51 => "BKSP".to_string(),
        53 => "ESC".to_string(),
        55 => "CMD".to_string(),
        56 => "SHIFT".to_string(),
        57 => "CAPS".to_string(),
        58 => "OPT".to_string(),
        59 => "CTRL".to_string(),
        60 => "SHIFT".to_string(),
        61 => "OPT".to_string(),
        62 => "CTRL".to_string(),
        63 => "FN".to_string(),
        64 => "F17".to_string(),
        65 => ".".to_string(),
        67 => "*".to_string(),
        69 => "+".to_string(),
        71 => "CLEAR".to_string(),
        75 => "/".to_string(),
        76 => "ENTER".to_string(),
        78 => "-".to_string(),
        79 => "F18".to_string(),
        80 => "F19".to_string(),
        81 => "=".to_string(),
        82 => "0".to_string(),
        83 => "1".to_string(),
        84 => "2".to_string(),
        85 => "3".to_string(),
        86 => "4".to_string(),
        87 => "5".to_string(),
        88 => "6".to_string(),
        89 => "7".to_string(),
        90 => "F20".to_string(),
        91 => "8".to_string(),
        92 => "9".to_string(),
        96 => "F5".to_string(),
        97 => "F6".to_string(),
        98 => "F7".to_string(),
        99 => "F3".to_string(),
        100 => "F8".to_string(),
        101 => "F9".to_string(),
        103 => "F11".to_string(),
        105 => "F13".to_string(),
        106 => "F16".to_string(),
        107 => "F14".to_string(),
        109 => "F10".to_string(),
        111 => "F12".to_string(),
        113 => "F15".to_string(),
        114 => "HELP".to_string(),
        115 => "HOME".to_string(),
        116 => "PG UP".to_string(),
        117 => "DEL".to_string(),
        118 => "F4".to_string(),
        119 => "END".to_string(),
        120 => "F2".to_string(),
        121 => "PG DN".to_string(),
        122 => "F1".to_string(),
        123 => "LEFT".to_string(),
        124 => "RIGHT".to_string(),
        125 => "DOWN".to_string(),
        126 => "UP".to_string(),
        _ => format!("KEY{}", keycode),
    }
}

// Get a unique code for each key to track press/release (Linux only)
#[cfg(target_os = "linux")]
fn key_to_code(key: Key) -> u32 {
    // Use hash of the debug representation as a unique identifier
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    format!("{:?}", key).hash(&mut hasher);
    hasher.finish() as u32
}

// Convert mouse button to label (Linux only)
#[cfg(target_os = "linux")]
fn button_to_label(button: Button) -> String {
    match button {
        Button::Left => "LMB".to_string(),
        Button::Right => "RMB".to_string(),
        Button::Middle => "MMB".to_string(),
        Button::Unknown(n) => format!("MB{}", n),
    }
}

// Get a unique code for each mouse button (Linux only)
#[cfg(target_os = "linux")]
fn button_to_code(button: Button) -> u32 {
    // Use a high base value to avoid collision with keyboard key codes
    const MOUSE_BUTTON_BASE: u32 = 0xFFFF0000;
    match button {
        Button::Left => MOUSE_BUTTON_BASE + 1,
        Button::Right => MOUSE_BUTTON_BASE + 2,
        Button::Middle => MOUSE_BUTTON_BASE + 3,
        Button::Unknown(n) => MOUSE_BUTTON_BASE + 100 + n as u32,
    }
}

#[cfg(target_os = "macos")]
pub fn start_keyboard_hook(state: Arc<RwLock<AppState>>) {
    eprintln!("[Keyboard Hook] Starting macOS CGEventTap listener...");
    
    use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
    use core_graphics::event::{CGEvent, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType, EventField, CGEventTapProxy};
    
    let state_ptr = Arc::into_raw(state) as *mut std::ffi::c_void;
    
    let callback = Box::new(move |_proxy: CGEventTapProxy, event_type: CGEventType, event: &CGEvent| -> Option<CGEvent> {
        let state = unsafe { Arc::from_raw(state_ptr as *const RwLock<AppState>) };
        let state_clone = Arc::clone(&state);
        std::mem::forget(state); // Don't drop the Arc
        
        match event_type {
            CGEventType::KeyDown => {
                let keycode = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
                eprintln!("[Keyboard Hook] Key down: keycode {}", keycode);
                
                // Check if target window matches
                let target_config = {
                    let state_lock = state_clone.read();
                    state_lock.target_config.clone()
                };
                
                eprintln!("[Keyboard Hook] Target config - mode: {}, value: {:?}", target_config.mode, target_config.value);
                
                if !should_process_event(&target_config) {
                    eprintln!("[Keyboard Hook] Event filtered out by target config");
                    return Some(event.to_owned());
                }
                
                // Convert keycode to label
                let label = keycode_to_label(keycode as u16);
                eprintln!("[Keyboard Hook] Adding key: {} (code: {})", label, keycode);
                
                let mut state_lock = state_clone.write();
                state_lock.add_key(keycode as u32, label);
            }
            CGEventType::KeyUp => {
                let keycode = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
                let mut state_lock = state_clone.write();
                state_lock.remove_key(keycode as u32);
            }
            _ => {}
        }
        
        Some(event.to_owned())
    });
    
    match CGEventTap::new(
        CGEventTapLocation::HID,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::ListenOnly,
        vec![CGEventType::KeyDown, CGEventType::KeyUp],
        callback,
    ) {
        Ok(tap) => {
            let loop_source = tap.mach_port.create_runloop_source(0).expect("Failed to create runloop source");
            
            unsafe {
                let run_loop = CFRunLoop::get_current();
                run_loop.add_source(&loop_source, kCFRunLoopCommonModes);
                tap.enable();
                
                eprintln!("[Keyboard Hook] CGEventTap enabled, starting runloop...");
                CFRunLoop::run_current();
            }
        }
        Err(()) => {
            eprintln!("[Keyboard Hook] Failed to create CGEventTap!");
            eprintln!("[Keyboard Hook] Make sure Accessibility permission is granted.");
        }
    }
}

// Windows: Pure polling-based detection (most reliable for games)
// NO HOOK INSTALLED - hooks can interfere with keyboard input!
#[cfg(target_os = "windows")]
pub fn start_keyboard_hook(state: Arc<RwLock<AppState>>) {
    eprintln!("[Keyboard] Windows: Starting pure polling-based key detection...");
    eprintln!("[Keyboard] NO hook installed - using GetAsyncKeyState only");
    eprintln!("[Keyboard] This method works reliably with all games and doesn't interfere with input");
    
    // Start polling thread - this is the ONLY method on Windows
    // No rdev hook is installed to avoid interfering with keyboard input
    validate_key_state_loop(state);
    
    // This function will block forever (polling loop)
    // If it returns, something went wrong
    eprintln!("[Keyboard] Polling loop ended unexpectedly!");
}

// Linux: Hook-based detection with async processing
#[cfg(target_os = "linux")]
pub fn start_keyboard_hook(state: Arc<RwLock<AppState>>) {
    eprintln!("[Keyboard Hook] Linux: Starting hook-based key detection...");
    
    let (tx, rx) = mpsc::channel::<InputEvent>();
    
    // Spawn event processor thread
    let state_for_processor = state.clone();
    std::thread::spawn(move || {
        eprintln!("[Event Processor] Started");
        process_input_events(rx, state_for_processor);
    });
    
    let callback = move |event: Event| {
        let input_event = match event.event_type {
            EventType::KeyPress(key) => {
                let code = key_to_code(key);
                let label = key_to_label(key);
                Some(InputEvent::KeyPress { code, label })
            }
            EventType::KeyRelease(key) => {
                let code = key_to_code(key);
                Some(InputEvent::KeyRelease { code })
            }
            EventType::ButtonPress(button) => {
                let code = button_to_code(button);
                let label = button_to_label(button);
                Some(InputEvent::ButtonPress { code, label })
            }
            EventType::ButtonRelease(button) => {
                let code = button_to_code(button);
                Some(InputEvent::ButtonRelease { code })
            }
            _ => None,
        };
        
        if let Some(evt) = input_event {
            let _ = tx.send(evt);
        }
    };

    eprintln!("[Keyboard Hook] Calling rdev::listen()...");
    if let Err(error) = listen(callback) {
        eprintln!("[Keyboard Hook] ERROR: {:?}", error);
    }
    eprintln!("[Keyboard Hook] Listener stopped unexpectedly");
}

// Process input events in a separate thread (Linux only)
#[cfg(target_os = "linux")]
fn process_input_events(rx: mpsc::Receiver<InputEvent>, state: Arc<RwLock<AppState>>) {
    // Cache for target config to reduce lock contention
    let mut cached_target_mode = String::new();
    let mut cached_target_value: Option<String> = None;
    let mut config_check_counter = 0u32;
    const CONFIG_REFRESH_INTERVAL: u32 = 50; // Refresh config every N events
    
    loop {
        // Wait for next event
        let event = match rx.recv() {
            Ok(e) => e,
            Err(_) => {
                eprintln!("[Event Processor] Channel closed, stopping");
                break;
            }
        };
        
        // Periodically refresh cached target config
        config_check_counter += 1;
        if config_check_counter >= CONFIG_REFRESH_INTERVAL || cached_target_mode.is_empty() {
            config_check_counter = 0;
            let state_lock = state.read();
            cached_target_mode = state_lock.target_config.mode.clone();
            cached_target_value = state_lock.target_config.value.clone();
        }
        
        match event {
            InputEvent::KeyPress { code, label } => {
                // Check if target window matches (uses cached config)
                if !should_process_event_cached(&cached_target_mode, &cached_target_value) {
                    continue;
                }
                
                eprintln!("[Event Processor] KeyPress: code={}, label={}", code, label);
                let mut state_lock = state.write();
                state_lock.add_key(code, label);
            }
            InputEvent::KeyRelease { code } => {
                // For release events: process if key is tracked OR mode is "all"
                let should_process = {
                    let state_lock = state.read();
                    state_lock.is_key_pressed(code) || cached_target_mode == "all"
                };
                
                if should_process {
                    eprintln!("[Event Processor] KeyRelease: code={}", code);
                    let mut state_lock = state.write();
                    state_lock.remove_key(code);
                }
            }
            InputEvent::ButtonPress { code, label } => {
                if !should_process_event_cached(&cached_target_mode, &cached_target_value) {
                    continue;
                }
                
                eprintln!("[Event Processor] ButtonPress: code={}, label={}", code, label);
                let mut state_lock = state.write();
                state_lock.add_key(code, label);
            }
            InputEvent::ButtonRelease { code } => {
                let should_process = {
                    let state_lock = state.read();
                    state_lock.is_key_pressed(code) || cached_target_mode == "all"
                };
                
                if should_process {
                    eprintln!("[Event Processor] ButtonRelease: code={}", code);
                    let mut state_lock = state.write();
                    state_lock.remove_key(code);
                }
            }
        }
    }
}

// Optimized version that uses cached config (reduces lock contention)
#[cfg(not(target_os = "macos"))]
fn should_process_event_cached(mode: &str, value: &Option<String>) -> bool {
    match mode {
        "disabled" => false,
        "all" => true,
        "title" | "process" | "hwnd" | "class" => {
            // Get foreground window info and check if it matches
            if let Some(window_info) = window_info::get_foreground_window() {
                match mode {
                    "title" => {
                        if let Some(v) = value {
                            window_info.title.to_lowercase().contains(&v.to_lowercase())
                        } else {
                            false
                        }
                    }
                    "process" => {
                        if let Some(v) = value {
                            window_info.process.to_lowercase() == v.to_lowercase()
                        } else {
                            false
                        }
                    }
                    "hwnd" => {
                        if let Some(v) = value {
                            window_info.hwnd == *v
                        } else {
                            false
                        }
                    }
                    "class" => {
                        if let Some(v) = value {
                            window_info.class.to_lowercase() == v.to_lowercase()
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            } else {
                false
            }
        }
        _ => false,
    }
}

#[cfg(target_os = "macos")]
fn should_process_event(target_config: &crate::state::TargetConfig) -> bool {
    let mode = target_config.mode.as_str();
    
    match mode {
        "disabled" => {
            eprintln!("[Keyboard Hook] Mode is 'disabled' - ignoring event");
            false
        },
        "all" => {
            eprintln!("[Keyboard Hook] Mode is 'all' - accepting event");
            true
        },
        "title" | "process" | "hwnd" | "class" => {
            // Get foreground window info and check if it matches
            if let Some(window_info) = window_info::get_foreground_window() {
                eprintln!("[Keyboard Hook] Checking window - title: '{}', process: '{}'", 
                         window_info.title, window_info.process);
                
                let result = match mode {
                    "title" => {
                        if let Some(value) = &target_config.value {
                            window_info.title.to_lowercase().contains(&value.to_lowercase())
                        } else {
                            false
                        }
                    }
                    "process" => {
                        if let Some(value) = &target_config.value {
                            window_info.process.to_lowercase() == value.to_lowercase()
                        } else {
                            false
                        }
                    }
                    "hwnd" => {
                        if let Some(value) = &target_config.value {
                            window_info.hwnd == *value
                        } else {
                            false
                        }
                    }
                    "class" => {
                        if let Some(value) = &target_config.value {
                            window_info.class.to_lowercase() == value.to_lowercase()
                        } else {
                            false
                        }
                    }
                    _ => false,
                };
                
                eprintln!("[Keyboard Hook] Filter result: {}", result);
                
                result
            } else {
                eprintln!("[Keyboard Hook] Could not get foreground window");
                false
            }
        }
        _ => {
            eprintln!("[Keyboard Hook] Unknown mode: {}", mode);
            false
        }
    }
}

// Windows: Pure polling-based key state detection using GetAsyncKeyState
// This is the PRIMARY method for detecting key states - Hook is just a backup
// Games using DirectInput/Raw Input work perfectly with this approach
#[cfg(target_os = "windows")]
fn validate_key_state_loop(state: Arc<RwLock<AppState>>) {
    use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
    
    // All keys to monitor with their VK codes
    // Using u32 as key code (VK code directly for polling)
    const MONITORED_KEYS: &[(i32, &str)] = &[
        // Letters A-Z
        (0x41, "A"), (0x42, "B"), (0x43, "C"), (0x44, "D"),
        (0x45, "E"), (0x46, "F"), (0x47, "G"), (0x48, "H"),
        (0x49, "I"), (0x4A, "J"), (0x4B, "K"), (0x4C, "L"),
        (0x4D, "M"), (0x4E, "N"), (0x4F, "O"), (0x50, "P"),
        (0x51, "Q"), (0x52, "R"), (0x53, "S"), (0x54, "T"),
        (0x55, "U"), (0x56, "V"), (0x57, "W"), (0x58, "X"),
        (0x59, "Y"), (0x5A, "Z"),
        // Numbers 0-9
        (0x30, "0"), (0x31, "1"), (0x32, "2"), (0x33, "3"),
        (0x34, "4"), (0x35, "5"), (0x36, "6"), (0x37, "7"),
        (0x38, "8"), (0x39, "9"),
        // Special keys
        (0x20, "SPACE"), (0x0D, "ENTER"), (0x09, "TAB"),
        (0x1B, "ESC"), (0x08, "BKSP"), (0x2E, "DEL"),
        (0x2D, "INS"), (0x24, "HOME"), (0x23, "END"),
        (0x21, "PG UP"), (0x22, "PG DN"),
        // Modifiers (use specific left/right for accurate tracking)
        (0xA0, "SHIFT"), // VK_LSHIFT
        (0xA1, "SHIFT"), // VK_RSHIFT  
        (0xA2, "CTRL"),  // VK_LCONTROL
        (0xA3, "CTRL"),  // VK_RCONTROL
        (0xA4, "ALT"),   // VK_LMENU
        (0xA5, "ALT"),   // VK_RMENU
        (0x14, "CAPS"),
        (0x5B, "WIN"),   // VK_LWIN
        (0x5C, "WIN"),   // VK_RWIN
        // Arrow keys
        (0x25, "LEFT"), (0x26, "UP"), (0x27, "RIGHT"), (0x28, "DOWN"),
        // Function keys
        (0x70, "F1"), (0x71, "F2"), (0x72, "F3"), (0x73, "F4"),
        (0x74, "F5"), (0x75, "F6"), (0x76, "F7"), (0x77, "F8"),
        (0x78, "F9"), (0x79, "F10"), (0x7A, "F11"), (0x7B, "F12"),
        // Mouse buttons
        (0x01, "LMB"), (0x02, "RMB"), (0x04, "MMB"),
        // Symbols
        (0xC0, "`"), (0xBD, "-"), (0xBB, "="),
        (0xDB, "["), (0xDD, "]"), (0xDC, "\\"),
        (0xBA, ";"), (0xDE, "'"),
        (0xBC, ","), (0xBE, "."), (0xBF, "/"),
        // Numpad
        (0x60, "NUM0"), (0x61, "NUM1"), (0x62, "NUM2"), (0x63, "NUM3"),
        (0x64, "NUM4"), (0x65, "NUM5"), (0x66, "NUM6"), (0x67, "NUM7"),
        (0x68, "NUM8"), (0x69, "NUM9"),
        (0x6A, "*"), (0x6B, "+"), (0x6D, "-"), (0x6E, "."), (0x6F, "/"),
    ];
    
    // Track which VK codes are currently "pressed" according to our state
    let mut polling_state: std::collections::HashMap<i32, bool> = std::collections::HashMap::new();
    
    eprintln!("[Key Poller] Starting pure polling mode ({}ms interval)", KEY_POLLING_INTERVAL_MS);
    
    loop {
        std::thread::sleep(Duration::from_millis(KEY_POLLING_INTERVAL_MS));
        
        // Check target window filter
        let target_config = {
            let s = state.read();
            s.target_config.clone()
        };
        
        // Skip if disabled
        if target_config.mode == "disabled" {
            // Clear all keys when disabled
            let has_keys = { state.read().key_labels.len() > 0 };
            if has_keys {
                state.write().clear_keys();
                polling_state.clear();
            }
            continue;
        }
        
        // Check if current window matches target (for non-"all" modes)
        let should_track = if target_config.mode == "all" {
            true
        } else {
            should_process_event_cached(&target_config.mode, &target_config.value)
        };
        
        if !should_track {
            // Clear keys when not in target window
            let has_keys = { state.read().key_labels.len() > 0 };
            if has_keys {
                state.write().clear_keys();
                polling_state.clear();
            }
            continue;
        }
        
        // Poll all monitored keys
        let mut keys_to_add: Vec<(i32, &str)> = Vec::new();
        let mut keys_to_remove: Vec<i32> = Vec::new();
        
        for &(vk, label) in MONITORED_KEYS {
            let key_state = unsafe { GetAsyncKeyState(vk) };
            let is_down = (key_state as u16 & 0x8000) != 0;
            let was_down = *polling_state.get(&vk).unwrap_or(&false);
            
            if is_down && !was_down {
                // Key just pressed
                eprintln!("[Poller] PRESS: {} (vk=0x{:02X}, raw_state=0x{:04X})", label, vk, key_state);
                keys_to_add.push((vk, label));
                polling_state.insert(vk, true);
            } else if !is_down && was_down {
                // Key just released
                eprintln!("[Poller] RELEASE: {} (vk=0x{:02X}, raw_state=0x{:04X})", label, vk, key_state);
                keys_to_remove.push(vk);
                polling_state.insert(vk, false);
            }
        }
        
        // Apply changes
        if !keys_to_add.is_empty() || !keys_to_remove.is_empty() {
            let mut state_lock = state.write();
            
            for (vk, label) in &keys_to_add {
                // Use VK code as the key code for polling-based tracking
                let code = *vk as u32 | 0x80000000; // High bit set to distinguish from hook codes
                state_lock.add_key(code, label.to_string());
            }
            
            for vk in &keys_to_remove {
                let code = *vk as u32 | 0x80000000;
                state_lock.remove_key(code);
            }
            
            // Debug: show current state
            let current_keys: Vec<String> = state_lock.label_order.iter().cloned().collect();
            eprintln!("[Poller] Current keys: {:?}", current_keys);
        }
    }
}
