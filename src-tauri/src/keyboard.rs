use crate::state::AppState;
use crate::window_info;
use parking_lot::RwLock;
use std::sync::Arc;

#[cfg(not(target_os = "macos"))]
use rdev::{listen, Event, EventType, Key};

#[cfg(target_os = "macos")]
use rdev::Key;

// Convert rdev::Key to a display label
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

// Get a unique code for each key to track press/release
#[cfg(not(target_os = "macos"))]
fn key_to_code(key: Key) -> u32 {
    // Use hash of the debug representation as a unique identifier
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    format!("{:?}", key).hash(&mut hasher);
    hasher.finish() as u32
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

#[cfg(not(target_os = "macos"))]
pub fn start_keyboard_hook(state: Arc<RwLock<AppState>>) {
    eprintln!("[Keyboard Hook] Starting keyboard event listener...");
    
    let callback = move |event: Event| {
        match event.event_type {
            EventType::KeyPress(key) => {
                // Check if target window matches
                let target_config = {
                    let state_lock = state.read();
                    state_lock.target_config.clone()
                };

                if !should_process_event(&target_config) {
                    return;
                }

                let code = key_to_code(key);
                let label = key_to_label(key);

                let mut state_lock = state.write();
                state_lock.add_key(code, label);
            }
            EventType::KeyRelease(key) => {
                let code = key_to_code(key);
                let mut state_lock = state.write();
                state_lock.remove_key(code);
            }
            _ => {}
        }
    };

    // Start listening for keyboard events (this blocks)
    eprintln!("[Keyboard Hook] Calling rdev::listen()...");
    if let Err(error) = listen(callback) {
        eprintln!("[Keyboard Hook] ERROR: {:?}", error);
    }
    eprintln!("[Keyboard Hook] Listener stopped unexpectedly");
}

fn should_process_event(target_config: &crate::state::TargetConfig) -> bool {
    let mode = target_config.mode.as_str();
    
    match mode {
        "disabled" => {
            #[cfg(target_os = "macos")]
            eprintln!("[Keyboard Hook] Mode is 'disabled' - ignoring event");
            false
        },
        "all" => {
            #[cfg(target_os = "macos")]
            eprintln!("[Keyboard Hook] Mode is 'all' - accepting event");
            true
        },
        "title" | "process" | "hwnd" | "class" => {
            // Get foreground window info and check if it matches
            if let Some(window_info) = window_info::get_foreground_window() {
                #[cfg(target_os = "macos")]
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
                
                #[cfg(target_os = "macos")]
                eprintln!("[Keyboard Hook] Filter result: {}", result);
                
                result
            } else {
                #[cfg(target_os = "macos")]
                eprintln!("[Keyboard Hook] Could not get foreground window");
                false
            }
        }
        _ => {
            #[cfg(target_os = "macos")]
            eprintln!("[Keyboard Hook] Unknown mode: {}", mode);
            false
        }
    }
}
