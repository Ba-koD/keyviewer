use crate::state::AppState;
use crate::window_info;
use parking_lot::RwLock;
use rdev::{listen, Event, EventType, Key};
use std::sync::Arc;

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
        Key::MetaLeft | Key::MetaRight => "WIN".to_string(),
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

// Get a unique code for each key to track press/release
fn key_to_code(key: Key) -> u32 {
    // Use hash of the debug representation as a unique identifier
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    format!("{:?}", key).hash(&mut hasher);
    hasher.finish() as u32
}

pub fn start_keyboard_hook(state: Arc<RwLock<AppState>>) {
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
    if let Err(error) = listen(callback) {
        eprintln!("Keyboard hook error: {:?}", error);
    }
}

fn should_process_event(target_config: &crate::state::TargetConfig) -> bool {
    match target_config.mode.as_str() {
        "disabled" => false,
        "all" => true,
        "title" | "process" | "hwnd" | "class" => {
            // Get foreground window info and check if it matches
            if let Some(window_info) = window_info::get_foreground_window() {
                match target_config.mode.as_str() {
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
                }
            } else {
                false
            }
        }
        _ => false,
    }
}

